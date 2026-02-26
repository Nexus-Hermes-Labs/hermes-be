# Hermes - TODO

> Architecture v2.1.0 baz alınarak hazırlanmıştır.

---

## Faz 0 — Messaging Servisi Refactor (Öncelikli)

### Chan + Chat → Messaging Service Birleştirme

- [ ] `channel-service` (8083) ve `chat-service` (8084) tek serviste birleştir → `messaging-service`
- [ ] Port tahsisi: `8083` veya yeni port belirle, Traefik config güncelle
- [ ] Mevcut channel CRUD endpoint'lerini koru
- [ ] Mevcut chat/message endpoint'lerini taşı
- [ ] WebSocket mantığını (şu an realtime-service üzerinden) messaging ile koordine et

### Veritabanı Şeması Güncellemeleri

- [ ] `conversations` tablosu oluştur (DM / Group DM)
  ```sql
  CREATE TABLE conversations (
      id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
      type       conversation_type NOT NULL, -- 'dm' | 'group_dm'
      created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  ```
- [ ] `conversation_members` tablosu oluştur
  ```sql
  CREATE TABLE conversation_members (
      conversation_id UUID REFERENCES conversations(id) ON DELETE CASCADE,
      user_id         UUID REFERENCES users(id) ON DELETE CASCADE,
      PRIMARY KEY (conversation_id, user_id)
  );
  ```
- [ ] `messages` tablosunu hem `channel_id` hem `conversation_id` destekleyecek şekilde güncelle
    - CHECK constraint: sadece biri dolu olmalı
- [ ] Gerekli migration'ları yaz ve test et

### Yeni Endpoint'ler

- [ ] `POST /v1/conversations` — DM / Group DM oluştur
- [ ] `GET /v1/conversations` — kullanıcının conversation listesi
- [ ] `GET /v1/conversations/:id/messages` — conversation mesaj geçmişi
- [ ] `DELETE /v1/conversations/:id` — conversation sil / üyeyi çıkar

### gRPC Güncellemeleri

- [ ] `channel-service` → `guild-service` permission check flow'u yeni servise taşı
- [ ] Proto dosyalarını güncelle (`proto/` dizini)
- [ ] `chat-service`'e ait proto tanımlarını messaging'e merge et

### Traefik Routing Güncellemesi

- [ ] `/v1/channels/*` → `messaging-service`
- [ ] `/v1/messages/*` → `messaging-service`
- [ ] `/v1/conversations/*` → `messaging-service` (yeni)
- [ ] Eski `chat-service` upstream'ini kaldır

### NATS Event Güncellemeleri

- [ ] `message.created`, `message.updated`, `message.deleted` event'larının yeni servisten yayınlandığını doğrula
- [ ] `conversation.created`, `conversation.member.joined` event'larını ekle
- [ ] realtime-service subscription'larını güncelle

---

## Faz 1 — Guild Servisi Temizliği

- [ ] Guild-service içinde kalmış mesajlaşmaya dair herhangi bir mantık var mı kontrol et
- [ ] Varsa messaging-service'e taşı
- [ ] Guild-service yalnızca guild metadata, roller, üyeler, davetler ile ilgilenmeli

---

## Faz 2 — Phase 2 Servisleri (Mevcut Planda Var)

- [ ] `presence-service` (8087) — online/offline/idle/DND, typing indicator
- [ ] `media-service` (8088) — dosya yükleme, resim işleme, CDN proxy, avatar
- [ ] `notification-service` (8089) — push bildirim, okunmamış sayısı, @mention

---

## Faz 3 — AI Servisi

- [ ] `ai-service` (8091) kur
- [ ] **Phase 3a**: Text translation pipeline
    - [ ] Dil tespiti
    - [ ] NATS: `translation.requested` → `translation.completed` akışı
    - [ ] Orijinal mesaj korunur, çeviriler ayrı saklanır
- [ ] **Phase 3b**: Voice STT (Speech-to-Text)
- [ ] **Phase 3c**: Voice TTS (Text-to-Speech) — hedef <500ms uçtan uca

---

## Faz 4 — Search & Voice Servisleri

### Search Service (8090)

- [ ] `search-service` kur
- [ ] Mesajlar, kullanıcılar, guild'ler için full-text search
- [ ] PostgreSQL FTS veya Meilisearch/Elasticsearch değerlendirmesi

### Voice Service (8085)

- [ ] `voice-service` kur — messaging'den **tamamen bağımsız** ayrı servis
- [ ] Media server seç: **Livekit** veya **mediasoup**
- [ ] WebRTC signaling endpoint'leri (offer/answer/ICE)
- [ ] Voice channel join / leave mantığı
- [ ] Yetki kontrolü için messaging-service'e gRPC ile sor ("kanal var mı, yetkisi var mı?")
- [ ] Mute / deafen durumu yönetimi
- [ ] ai-service entegrasyonu (Phase 3b/3c için audio stream)

---

## Genel / Altyapı

- [ ] Arch dokümanını servis değişikliklerini yansıtacak şekilde güncelle (v2.2.0)
- [ ] Her servisin port ve sorumluluk tablosunu güncel tut
- [ ] Shared DB'den database-per-service geçiş planı hazırla (Post-MVP)
- [ ] CI/CD pipeline'ına yeni servis ekle (GitHub Actions)
- [ ] Prometheus metric'lerini yeni servisler için güncelle

---

## Öncelik Sırası

```
Faz 0 (Messaging Refactor)
    → Faz 1 (Guild Temizliği)
        → Faz 2 (Presence / Media / Notification)
            → Faz 3 (AI)
                → Faz 4 (Search + Voice)
```