use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::application::{GuildInviteService, GuildMemberService, GuildRoleService, GuildService};
use crate::presentation::grpc::proto::guild::v1::guild_service_server::GuildService as GuildServiceTrait;
use crate::presentation::grpc::proto::guild::v1::{
    CreateGuildRequest, CreateInviteRequest, DeleteGuildRequest, GetGuildMemberRequest,
    GetGuildMembersRequest, GetGuildMembersResponse, GetGuildRequest, GetInviteRequest,
    GuildMemberResponse, GuildResponse, InviteResponse, IsGuildMemberRequest,
    IsGuildMemberResponse, RemoveGuildMemberRequest, RevokeInviteRequest, UpdateGuildRequest,
    UseInviteRequest,
};

/// gRPC server implementation for `GuildService`
#[derive(Debug)]
#[allow(clippy::struct_field_names)]
pub struct GuildServiceGrpc {
    /// Guild management service
    guild_service: Arc<GuildService>,
    /// Member management service
    member_service: Arc<GuildMemberService>,
    #[allow(dead_code)]
    /// Role management service
    role_service: Arc<GuildRoleService>,
    /// Invite management service
    invite_service: Arc<GuildInviteService>,
}

impl GuildServiceGrpc {
    /// Create a new `GuildServiceGrpc` instance.
    #[must_use]
    pub const fn new(
        guild_service: Arc<GuildService>,
        member_service: Arc<GuildMemberService>,
        role_service: Arc<GuildRoleService>,
        invite_service: Arc<GuildInviteService>,
    ) -> Self {
        Self {
            guild_service,
            member_service,
            role_service,
            invite_service,
        }
    }
}

// ============================================
// CONVERTERS
// ============================================

fn guild_to_proto(g: &crate::domain::Guild) -> GuildResponse {
    GuildResponse {
        guild_id: g.id().to_string(),
        owner_id: g.owner_id().to_string(),
        name: g.name().as_str().to_string(),
        description: g.description().map(String::from),
        icon_url: g.icon_url().map(String::from),
        banner_url: g.banner_url().map(String::from),
        member_count: g.member_count(),
        created_at: Some(prost_types::Timestamp {
            seconds: g.created_at().timestamp(),
            nanos: g.created_at().timestamp_subsec_nanos().cast_signed(),
        }),
        updated_at: Some(prost_types::Timestamp {
            seconds: g.updated_at().timestamp(),
            nanos: g.updated_at().timestamp_subsec_nanos().cast_signed(),
        }),
    }
}

fn member_to_proto(m: &crate::domain::GuildMember) -> GuildMemberResponse {
    GuildMemberResponse {
        guild_id: m.guild_id().to_string(),
        user_id: m.user_id().to_string(),
        nickname: m
            .nickname()
            .map(|n| n.as_str().to_string())
            .unwrap_or_default(),
        role_ids: m.role_ids().iter().map(ToString::to_string).collect(),
        joined_at: Some(prost_types::Timestamp {
            seconds: m.joined_at().timestamp(),
            nanos: m.joined_at().timestamp_subsec_nanos().cast_signed(),
        }),
    }
}

fn invite_to_proto(inv: &crate::domain::GuildInvite) -> InviteResponse {
    InviteResponse {
        code: inv.code().as_str().to_string(),
        guild_id: inv.guild_id().to_string(),
        creator_id: inv.creator_id().to_string(),
        max_uses: inv.max_uses(),
        uses: inv.uses(),
        expires_at: inv.expires_at().map(|t| prost_types::Timestamp {
            seconds: t.timestamp(),
            nanos: t.timestamp_subsec_nanos().cast_signed(),
        }),
        created_at: Some(prost_types::Timestamp {
            seconds: inv.created_at().timestamp(),
            nanos: inv.created_at().timestamp_subsec_nanos().cast_signed(),
        }),
    }
}

// ============================================
// gRPC IMPLEMENTATION
// ============================================

#[tonic::async_trait]
impl GuildServiceTrait for GuildServiceGrpc {
    async fn create_guild(
        &self,
        request: Request<CreateGuildRequest>,
    ) -> Result<Response<GuildResponse>, Status> {
        let req = request.into_inner();
        let owner_id = Uuid::parse_str(&req.owner_id)
            .map_err(|_| Status::invalid_argument("Invalid owner_id"))?;

        let guild = self
            .guild_service
            .create_guild(owner_id, req.name, req.description)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(guild_to_proto(&guild)))
    }

    async fn get_guild(
        &self,
        request: Request<GetGuildRequest>,
    ) -> Result<Response<GuildResponse>, Status> {
        let req = request.into_inner();
        let guild_id = Uuid::parse_str(&req.guild_id)
            .map_err(|_| Status::invalid_argument("Invalid guild_id"))?;

        let guild = self
            .guild_service
            .get_guild(guild_id)
            .await
            .map_err(|e| Status::not_found(e.to_string()))?;

        Ok(Response::new(guild_to_proto(&guild)))
    }

    async fn update_guild(
        &self,
        request: Request<UpdateGuildRequest>,
    ) -> Result<Response<GuildResponse>, Status> {
        let req = request.into_inner();
        let guild_id = Uuid::parse_str(&req.guild_id)
            .map_err(|_| Status::invalid_argument("Invalid guild_id"))?;
        let requester_id = Uuid::parse_str(&req.requester_id)
            .map_err(|_| Status::invalid_argument("Invalid requester_id"))?;

        let guild = self
            .guild_service
            .update_guild(
                guild_id,
                requester_id,
                req.name,
                req.description,
                req.icon_url,
                req.banner_url,
                None,
            )
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(guild_to_proto(&guild)))
    }

    async fn delete_guild(
        &self,
        request: Request<DeleteGuildRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();
        let guild_id = Uuid::parse_str(&req.guild_id)
            .map_err(|_| Status::invalid_argument("Invalid guild_id"))?;
        let requester_id = Uuid::parse_str(&req.requester_id)
            .map_err(|_| Status::invalid_argument("Invalid requester_id"))?;

        self.guild_service
            .delete_guild(guild_id, requester_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(()))
    }

    async fn get_guild_members(
        &self,
        request: Request<GetGuildMembersRequest>,
    ) -> Result<Response<GetGuildMembersResponse>, Status> {
        let req = request.into_inner();
        let guild_id = Uuid::parse_str(&req.guild_id)
            .map_err(|_| Status::invalid_argument("Invalid guild_id"))?;

        let (limit, offset) = req.pagination.as_ref().map_or((50_i64, 0_i64), |p| {
            let page = p.page.max(1);
            let page_size = p.page_size.clamp(1, 100);
            (i64::from(page_size), i64::from((page - 1) * page_size))
        });

        let members = self
            .member_service
            .list_members(guild_id, limit, offset)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_members: Vec<GuildMemberResponse> = members.iter().map(member_to_proto).collect();

        let pagination_response = req.pagination.map(|p| {
            let page_size = p.page_size.clamp(1, 100);
            crate::presentation::grpc::proto::common::v1::PaginationResponse {
                total_items: i32::try_from(proto_members.len()).unwrap_or(i32::MAX),
                total_pages: 1,
                current_page: p.page.max(1),
                page_size,
            }
        });

        Ok(Response::new(GetGuildMembersResponse {
            members: proto_members,
            pagination: pagination_response,
        }))
    }

    async fn get_guild_member(
        &self,
        request: Request<GetGuildMemberRequest>,
    ) -> Result<Response<GuildMemberResponse>, Status> {
        let req = request.into_inner();
        let guild_id = Uuid::parse_str(&req.guild_id)
            .map_err(|_| Status::invalid_argument("Invalid guild_id"))?;
        let user_id = Uuid::parse_str(&req.user_id)
            .map_err(|_| Status::invalid_argument("Invalid user_id"))?;

        let member = self
            .member_service
            .get_member(guild_id, user_id)
            .await
            .map_err(|e| Status::not_found(e.to_string()))?;

        Ok(Response::new(member_to_proto(&member)))
    }

    async fn is_guild_member(
        &self,
        request: Request<IsGuildMemberRequest>,
    ) -> Result<Response<IsGuildMemberResponse>, Status> {
        let req = request.into_inner();
        let guild_id = Uuid::parse_str(&req.guild_id)
            .map_err(|_| Status::invalid_argument("Invalid guild_id"))?;
        let user_id = Uuid::parse_str(&req.user_id)
            .map_err(|_| Status::invalid_argument("Invalid user_id"))?;

        let is_member = self
            .member_service
            .is_member(guild_id, user_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(IsGuildMemberResponse { is_member }))
    }

    async fn remove_guild_member(
        &self,
        request: Request<RemoveGuildMemberRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();
        let guild_id = Uuid::parse_str(&req.guild_id)
            .map_err(|_| Status::invalid_argument("Invalid guild_id"))?;
        let user_id = Uuid::parse_str(&req.user_id)
            .map_err(|_| Status::invalid_argument("Invalid user_id"))?;
        let requester_id = Uuid::parse_str(&req.requester_id)
            .map_err(|_| Status::invalid_argument("Invalid requester_id"))?;

        self.member_service
            .kick_member(guild_id, user_id, requester_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(()))
    }

    async fn create_invite(
        &self,
        request: Request<CreateInviteRequest>,
    ) -> Result<Response<InviteResponse>, Status> {
        let req = request.into_inner();
        let guild_id = Uuid::parse_str(&req.guild_id)
            .map_err(|_| Status::invalid_argument("Invalid guild_id"))?;
        let creator_id = Uuid::parse_str(&req.creator_id)
            .map_err(|_| Status::invalid_argument("Invalid creator_id"))?;

        let invite = self
            .invite_service
            .create_invite(
                guild_id,
                creator_id,
                req.max_uses,
                req.max_age_seconds.map(i64::from),
            )
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(invite_to_proto(&invite)))
    }

    async fn get_invite(
        &self,
        request: Request<GetInviteRequest>,
    ) -> Result<Response<InviteResponse>, Status> {
        let req = request.into_inner();
        let invite = self
            .invite_service
            .get_invite(req.code)
            .await
            .map_err(|e| Status::not_found(e.to_string()))?;

        Ok(Response::new(invite_to_proto(&invite)))
    }

    async fn use_invite(
        &self,
        request: Request<UseInviteRequest>,
    ) -> Result<Response<GuildMemberResponse>, Status> {
        let req = request.into_inner();
        let user_id = Uuid::parse_str(&req.user_id)
            .map_err(|_| Status::invalid_argument("Invalid user_id"))?;

        let member = self
            .invite_service
            .use_invite(req.code, user_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(member_to_proto(&member)))
    }

    async fn revoke_invite(
        &self,
        request: Request<RevokeInviteRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();
        let requester_id = Uuid::parse_str(&req.requester_id)
            .map_err(|_| Status::invalid_argument("Invalid requester_id"))?;

        self.invite_service
            .revoke_invite(req.code, requester_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(()))
    }
}
