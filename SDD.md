# OpenIM SDK Core - Specification-Driven Development Document

> **Project**: OpenIM SDK Core v3  
> **Language**: Go 1.24.7  
> **License**: Apache-2.0  
> **Generated**: 2026-05-24

---

## 目录

- [1. 项目概述](#1-项目概述)
- [2. 架构规范](#2-架构规范)
- [3. 模块规范](#3-模块规范)
  - [3.1 连接管理模块](#31-连接管理模块-interaction)
  - [3.2 会话消息模块](#32-会话消息模块-conversation_msg)
  - [3.3 群组模块](#33-群组模块-group)
  - [3.4 关系链模块](#34-关系链模块-relation)
  - [3.5 用户模块](#35-用户模块-user)
  - [3.6 第三方服务模块](#36-第三方服务模块-third)
  - [3.7 在线状态模块](#37-在线状态模块-online)
  - [3.8 数据库模块](#38-数据库模块-db)
  - [3.9 回调接口规范](#39-回调接口规范)
- [4. 数据结构规范](#4-数据结构规范)
- [5. 接口规范](#5-接口规范)
  - [5.1 SDK 初始化与登录](#51-sdk-初始化与登录)
  - [5.2 用户接口](#52-用户接口)
  - [5.3 好友接口](#53-好友接口)
  - [5.4 群组接口](#54-群组接口)
  - [5.5 消息接口](#55-消息接口)
  - [5.6 会话接口](#56-会话接口)
  - [5.7 第三方服务接口](#57-第三方服务接口)
  - [5.8 在线状态接口](#58-在线状态接口)
- [6. 同步策略规范](#6-同步策略规范)
- [7. 错误码规范](#7-错误码规范)
- [8. 平台适配规范](#8-平台适配规范)
- [9. 性能规范](#9-性能规范)
- [10. 安全规范](#10-安全规范)
- [11. 扩展规范](#11-扩展规范)
- [12. 测试规范](#12-测试规范)
- [13. 可执行 Task 清单](#13-可执行-task-清单)
- [附录 A. 依赖清单](#附录)
- [附录 B. 关键文件索引](#b-关键文件索引)
- [附录 C. 核心模块详细实现指南](#附录-c-核心模块详细实现指南)
  - [C.1 SDK 初始化与登录实现](#c1-sdk-初始化与登录实现)
  - [C.2 WebSocket 长连接管理实现](#c2-websocket-长连接管理实现)
  - [C.3 消息同步机制实现](#c3-消息同步机制实现)
  - [C.4 数据库模块实现](#c4-数据库模块实现)
  - [C.5 会话消息处理实现](#c5-会话消息处理实现)
  - [C.6 Syncer 数据同步框架实现](#c6-syncer-数据同步框架实现)
  - [C.7 第三方服务模块实现](#c7-第三方服务模块实现)
  - [C.8 在线状态模块实现](#c8-在线状态模块实现)
- [附录 D. 功能完整性验证](#附录-d-功能完整性验证)
  - [D.1 openim-sdk-core 功能覆盖清单](#d1-openim-sdk-core-功能覆盖清单)
  - [D.2 openim-flutter-demo 功能覆盖清单](#d2-openim-flutter-demo-功能覆盖清单)
  - [D.3 核心流程验证](#d3-核心流程验证)
  - [D.4 未覆盖功能说明](#d4-未覆盖功能说明)

---

## 1. 项目概述

### 1.1 项目定位
OpenIM SDK Core 是 OpenIM 即时通讯系统的**核心跨平台 SDK**，为 iOS、Android、PC、Web (WebAssembly) 等所有平台提供统一的 IM 能力基础层。

### 1.2 核心能力
- WebSocket 长连接管理与智能心跳
- 消息编解码与多端同步
- 本地消息存储（SQLite）
- 关系链数据同步（好友/群组/黑名单）
- 跨平台通信与回调管理

---

## 2. 架构规范

### 2.1 分层架构

```
┌─────────────────────────────────────────────────────────┐
│                    OpenIM SDK API Layer                  │
│              (open_im_sdk/*.go - 对外接口)               │
├─────────────────────────────────────────────────────────┤
│              Internal Business Logic Layer               │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐   │
│  │conversation│ │  group   │ │ relation │ │   user   │   │
│  │   _msg   │ │          │ │          │ │          │   │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘   │
│  ┌──────────────────────────────────────────────────┐   │
│  │              interaction (长连接管理)              │   │
│  └──────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────┤
│                  Infrastructure Layer                    │
│  ┌────────┐ ┌────────┐ ┌───────┐ ┌────────┐ ┌────────┐ │
│  │  db    │ │ cache  │ │ syncer│ │  api   │ │ utils  │ │
│  └────────┘ └────────┘ └───────┘ └────────┘ └────────┘ │
├─────────────────────────────────────────────────────────┤
│                  External Dependencies                   │
│     WebSocket │ SQLite │ Protocol Buffers │ HTTP Client  │
└─────────────────────────────────────────────────────────┘
```

### 2.2 核心设计模式

#### 2.2.1 UserContext 聚合根
```go
// 全局单例，聚合所有业务模块
type UserContext struct {
    relation     *relation.Relation
    group        *group.Group
    conversation *conv.Conversation
    user         *user.User
    file         *file.File
    db           db_interface.DataBase
    longConnMgr  *interaction.LongConnMgr
    msgSyncer    *interaction.MsgSyncer
    third        *third.Third
    // ... 其他字段
}
```

**规范要点**:
- 全局唯一实例 `IMUserContext`
- 通过 `InitSDK()` 初始化配置
- 通过 `Login()` 建立连接并启动所有子模块
- 通过 `Logout()` 清理资源并重置状态

#### 2.2.2 Syncer 数据同步器
```go
// 泛型同步器，支持 Insert/Delete/Update/Notice 生命周期
type Syncer[T any, Resp any, Key any] struct {
    // 支持全量同步、增量同步、版本控制
}
```

**规范要点**:
- 每个业务模块（Group/Relation/Conversation）拥有独立 Syncer
- 支持 `WithInsert`, `WithDelete`, `WithUpdate`, `WithNotice` 回调
- 支持分页批量同步 `WithBatchPageReq`
- 支持本地缓存 `WithCache`

#### 2.2.3 事件驱动架构
```go
// 命令通道模式
type Cmd2Value struct {
    Ctx   context.Context
    Value interface{}
}

// 各模块通过 channel 通信
conversationEventQueue chan Cmd2Value
msgSyncerCh            chan Cmd2Value
loginMgrCh             chan Cmd2Value
```

---

## 3. 模块规范

### 3.1 连接管理模块 (interaction)

#### 3.1.1 LongConnMgr 长连接管理器

**职责**: WebSocket 连接生命周期管理、消息收发、心跳保活、断线重连

```go
type LongConnMgr struct {
    conn       LongConn          // WebSocket 连接
    send       chan Message      // 发送队列
    listener   func() open_im_sdk_callback.OnConnListener
    Syncer     *WsRespAsyn       // 异步响应同步器
    encoder    Encoder           // 消息编码器 (Gob)
    compressor Compressor        // 消息压缩器 (Gzip)
    sub        *subscription     // 用户在线状态订阅
    mb         *MessageBatcher   // 消息批处理器
}
```

**连接状态机**:
```
DefaultNotConnect → Connecting → Connected → Closed
                                    ↓
                              断线 → Reconnecting (最多300次)
```

**心跳策略**:
- `pongWait = 30s` - 等待服务端 pong 超时
- `pingPeriod = 24s` - 发送 ping 间隔 (pongWait * 8 / 10)
- `writeWait = 10s` - 写入超时

**消息处理流程**:
```
readPump() → handleMessage() → doPushMsg() → conversationEventQueue
writePump() ← send channel ← SendReqWaitResp()
```

#### 3.1.2 消息协议

**WebSocket 消息标识**:
| 标识 | 值 | 用途 |
|------|-----|------|
| GetNewestSeq | 1001 | 获取最新序列号 |
| PullMsgByRange | 1002 | 拉取消息范围 |
| SendMsg | 1003 | 发送消息 |
| PushMsg | 2001 | 推送消息 |
| KickOnlineMsg | 2002 | 踢下线通知 |
| LogoutMsg | 2003 | 登出通知 |

### 3.2 会话消息模块 (conversation_msg)

#### 3.2.1 Conversation 会话管理器

**职责**: 会话管理、消息收发、消息同步、本地存储

```go
type Conversation struct {
    *interaction.LongConnMgr
    db                          db_interface.DataBase
    conversationSyncer          *syncer.Syncer[*LocalConversation, ...]
    msgListener                 func() OnAdvancedMsgListener
    conversationEventQueue      chan Cmd2Value
    cache                       *cache.Cache[string, *LocalConversation]
    maxSeqRecorder              MaxSeqRecorder
    sender                      *messageSender
    typing                      *typing
}
```

**消息类型体系**:
```go
// 基础消息类型 (100-999)
Text     = 101    // 文本
Picture  = 102    // 图片
Sound    = 103    // 语音
Video    = 104    // 视频
File     = 105    // 文件
AtText   = 106    // @消息
Card     = 108    // 名片
Location = 109    // 位置
Custom   = 110    // 自定义
Typing   = 113    // 正在输入
Quote    = 114    // 引用

// 通知消息类型 (1000+)
FriendNotificationBegin = 1200
GroupNotificationBegin  = 1500
RevokeNotification      = 2101  // 撤回
HasReadReceipt          = 2200  // 已读回执
```

**消息状态**:
```go
MsgStatusSending     = 1  // 发送中
MsgStatusSendSuccess = 2  // 发送成功
MsgStatusSendFailed  = 3  // 发送失败
MsgStatusHasDeleted  = 4  // 已删除
```

**会话类型**:
```go
SingleChatType     = 1  // 单聊
WriteGroupChatType = 2  // 写扩散群聊
ReadGroupChatType  = 3  // 读扩散群聊
NotificationChatType = 4 // 通知会话
```

#### 3.2.2 消息同步策略

**全量同步** (重装后):
1. 从服务端拉取所有会话
2. 根据 maxSeq 拉取历史消息
3. 本地增量插入

**增量同步** (日常):
1. WebSocket 推送新消息
2. 按 conversationID 分组处理
3. 区分自己发送/他人发送的消息
4. 更新会话列表和未读数

### 3.3 群组模块 (group)

#### 3.3.1 Group 群组管理器

```go
type Group struct {
    listener               func() OnGroupListener
    db                     db_interface.DataBase
    groupSyncer            *syncer.Syncer[*LocalGroup, ...]
    groupMemberSyncer      *syncer.Syncer[*LocalGroupMember, ...]
    conversationEventQueue chan Cmd2Value
    groupInfoCache         *cache.Cache[string, *LocalGroup]
    groupMemberCache       *cache.Cache[string, *LocalGroupMember]
}
```

**群组角色**:
```go
GroupOwner         = 100  // 群主
GroupAdmin         = 60   // 管理员
GroupOrdinaryUsers = 20   // 普通成员
```

**群组状态**:
```go
GroupOk              = 0  // 正常
GroupBanChat         = 1  // 禁言
GroupStatusDismissed = 2  // 已解散
```

**群组类型**:
```go
NormalGroup  = 0  // 普通群
SuperGroup   = 1  // 超级群
WorkingGroup = 2  // 工作群
```

### 3.4 关系链模块 (relation)

#### 3.4.1 Relation 关系管理器

```go
type Relation struct {
    friendshipListener     OnFriendshipListenerSdk
    db                     db_interface.DataBase
    friendSyncer           *syncer.Syncer[*LocalFriend, ...]
    blackSyncer            *syncer.Syncer[*LocalBlack, ...]
    conversationEventQueue chan Cmd2Value
}
```

**关系类型**:
```go
BlackRelationship  = 0  // 黑名单
FriendRelationship = 1  // 好友
```

**好友申请响应**:
```go
FriendResponseAgree   = 1   // 同意
FriendResponseRefuse  = -1  // 拒绝
FriendResponseDefault = 0   // 默认
```

### 3.5 用户模块 (user)

#### 3.5.1 User 用户管理器

```go
type User struct {
    db_interface.DataBase
    loginUserID            string
    listener               func() OnUserListener
    userSyncer             *syncer.Syncer[*LocalUser, ...]
    userCache              *cache.UserCache[string, *LocalUser]
}
```

### 3.6 第三方服务模块 (third)

#### 3.6.1 Third 第三方服务管理器

**职责**: 文件上传、日志上传、FCM Token 管理、应用角标设置

```go
type Third struct {
    platform      int32
    loginUserID   string
    appFramework  string
    LogFilePath   string
    fileUploader  *file.File
    logUploadLock sync.Mutex
}
```

**核心功能**:

| 功能 | 方法 | 描述 |
|------|------|------|
| 文件上传 | `UploadFile` | 上传图片/视频/音频/文件到 OSS |
| 日志上传 | `UploadLogs` | 上传本地日志文件用于问题排查 |
| FCM Token | `UpdateFcmToken` | 更新 Firebase Cloud Messaging Token |
| 应用角标 | `SetAppBadge` | 设置应用未读消息角标数 |
| 日志记录 | `Log` | 客户端日志上报 |

#### 3.6.2 文件上传模块 (file)

**职责**: 文件分片上传、进度回调、图片压缩、MD5 计算

```go
type File struct {
    // 文件上传核心逻辑
}

type ReadFile interface {
    io.Reader
    io.Closer
    Size() int64
    StartSeek(whence int) error
}
```

**上传流程**:
```
1. 计算文件 MD5
2. 检查是否已存在（秒传）
3. 初始化上传任务
4. 分片上传（支持断点续传）
5. 上传完成回调
```

**进度回调**:
```go
type UploadFileCallback interface {
    Open(size int64)           // 打开文件
    PartComplete(index, size)  // 分片完成
    Complete(size)             // 上传完成
    HashProgress(current, total) // MD5 计算进度
}
```

### 3.7 在线状态模块 (online)

#### 3.7.1 用户在线状态订阅

**职责**: 订阅用户在线状态、获取用户在线平台列表

**核心功能**:

| 功能 | 方法 | 描述 |
|------|------|------|
| 订阅状态 | `SubscribeUsersStatus` | 订阅指定用户的在线状态 |
| 取消订阅 | `UnsubscribeUsersStatus` | 取消订阅用户在线状态 |
| 获取状态 | `GetSubscribeUsersStatus` | 获取已订阅用户的在线状态 |
| 获取平台 | `GetUserOnlinePlatformIDs` | 获取用户在线的平台 ID 列表 |

**在线状态结构**:
```go
type OnlineStatus struct {
    UserID      string  // 用户ID
    PlatformIDs []int32 // 在线平台ID列表
    Status      int     // 在线状态 (1=在线, 0=离线)
}
```

**平台在线标识**:
```go
const (
    Offline = 0  // 离线
    Online  = 1  // 在线
)
```

### 3.8 数据库模块 (db)

#### 3.8.1 数据库接口规范

```go
type DataBase interface {
    Close(ctx context.Context) error
    InitDB(ctx context.Context, userID string, dataDir string) error
    GroupModel
    MessageModel
    ConversationModel
    UserModel
    FriendModel
    S3Model
    SendingMessagesModel
    VersionSyncModel
    AppSDKVersion
    TableMaster
}
```

**存储引擎**: SQLite (gorm.io/driver/sqlite)

**核心数据模型**:
- `LocalChatLog` - 本地消息记录
- `LocalConversation` - 本地会话
- `LocalFriend` - 好友信息
- `LocalGroup` - 群组信息
- `LocalGroupMember` - 群成员信息
- `LocalBlack` - 黑名单
- `LocalUser` - 用户信息

### 3.9 回调接口规范

#### 3.9.1 连接回调
```go
type OnConnListener interface {
    OnConnecting()
    OnConnectSuccess()
    OnConnectFailed(errCode int32, errMsg string)
    OnKickedOffline()
    OnUserTokenExpired()
    OnUserTokenInvalid(errMsg string)
}
```

#### 3.7.2 消息回调
```go
type OnAdvancedMsgListener interface {
    OnRecvNewMessage(message string)
    OnRecvC2CReadReceipt(msgReceiptList string)
    OnNewRecvMessageRevoked(messageRevoked string)
    OnRecvOfflineNewMessage(message string)
    OnMsgDeleted(message string)
    OnRecvOnlineOnlyMessage(message string)
}
```

#### 3.7.3 会话回调
```go
type OnConversationListener interface {
    OnSyncServerStart(reinstalled bool)
    OnSyncServerFinish(reinstalled bool)
    OnSyncServerProgress(progress int)
    OnSyncServerFailed(reinstalled bool)
    OnNewConversation(conversationList string)
    OnConversationChanged(conversationList string)
    OnTotalUnreadMessageCountChanged(totalUnreadCount int32)
    OnConversationUserInputStatusChanged(change string)
}
```

---

## 4. 数据结构规范

### 4.1 消息结构 (MsgStruct)

```go
type MsgStruct struct {
    ClientMsgID      string    `json:"clientMsgID"`      // 客户端消息ID
    ServerMsgID      string    `json:"serverMsgID"`      // 服务端消息ID
    CreateTime       int64     `json:"createTime"`       // 创建时间
    SendTime         int64     `json:"sendTime"`         // 发送时间
    SessionType      int32     `json:"sessionType"`      // 会话类型
    SendID           string    `json:"sendID"`           // 发送者ID
    RecvID           string    `json:"recvID"`           // 接收者ID
    MsgFrom          int32     `json:"msgFrom"`          // 消息来源
    ContentType      int32     `json:"contentType"`      // 内容类型
    SenderPlatformID int32     `json:"senderPlatformID"` // 发送者平台
    SenderNickname   string    `json:"senderNickname"`   // 发送者昵称
    SenderFaceURL    string    `json:"senderFaceUrl"`    // 发送者头像
    GroupID          string    `json:"groupID"`          // 群组ID
    Content          string    `json:"content"`          // 消息内容(JSON)
    Seq              int64     `json:"seq"`              // 序列号
    IsRead           bool      `json:"isRead"`           // 是否已读
    Status           int32     `json:"status"`           // 状态
    OfflinePush      *OfflinePushInfo `json:"offlinePush"` // 离线推送
    Ex               string    `json:"ex"`               // 扩展字段
    LocalEx          string    `json:"localEx"`          // 本地扩展
    // 具体消息内容字段
    TextElem         *TextElem
    PictureElem      *PictureElem
    SoundElem        *SoundElem
    VideoElem        *VideoElem
    FileElem         *FileElem
    // ... 更多类型
}
```

### 4.2 SDK 配置结构 (IMConfig)

```go
type IMConfig struct {
    SystemType          string `json:"systemType"`           // 系统类型
    PlatformID          int32  `json:"platformID"`           // 平台ID
    ApiAddr             string `json:"apiAddr"`              // API地址
    WsAddr              string `json:"wsAddr"`               // WebSocket地址
    DataDir             string `json:"dataDir"`              // 数据目录
    LogLevel            uint32 `json:"logLevel"`             // 日志级别
    IsLogStandardOutput bool   `json:"isLogStandardOutput"`  // 标准输出
    LogFilePath         string `json:"logFilePath"`          // 日志路径
    LogRemainCount      uint32 `json:"logRemainCount"`       // 日志保留数
    StopGoroutineOnBackground bool `json:"stopGoroutineOnBackground"` // 后台停止协程
}
```

---

## 5. 接口规范

### 5.1 SDK 初始化与登录

| 方法 | 参数 | 说明 |
|------|------|------|
| `InitSDK` | listener, operationID, config | 初始化SDK |
| `UnInitSDK` | operationID | 反初始化SDK |
| `Login` | callback, operationID, userID, token | 登录 |
| `Logout` | callback, operationID | 登出 |
| `GetLoginStatus` | operationID | 获取登录状态 |
| `GetLoginUserID` | - | 获取当前登录用户ID |
| `SetAppBackgroundStatus` | callback, operationID, isBackground | 设置应用后台状态 |
| `NetworkStatusChanged` | callback, operationID | 网络状态变化 |

### 5.2 用户接口

| 方法 | 参数 | 说明 |
|------|------|------|
| `GetUsersInfo` | callback, operationID, userIDs | 获取用户信息 |
| `GetSelfUserInfo` | callback, operationID | 获取自己的信息 |
| `SetSelfInfo` | callback, operationID, userInfo | 设置自己的信息 |
| `GetUserClientConfig` | callback, operationID | 获取用户客户端配置 |

### 5.3 好友接口

| 方法 | 参数 | 说明 |
|------|------|------|
| `GetFriendList` | callback, operationID, filterBlack | 获取好友列表 |
| `GetFriendListPage` | callback, operationID, offset, count, filterBlack | 分页获取好友 |
| `AddFriend` | callback, operationID, userIDReqMsg | 添加好友 |
| `DeleteFriend` | callback, operationID, friendUserID | 删除好友 |
| `AcceptFriendApplication` | callback, operationID, userIDHandleMsg | 接受好友申请 |
| `RefuseFriendApplication` | callback, operationID, userIDHandleMsg | 拒绝好友申请 |
| `AddBlack` | callback, operationID, blackUserID, ex | 添加黑名单 |
| `GetBlackList` | callback, operationID | 获取黑名单 |
| `RemoveBlack` | callback, operationID, removeUserID | 移除黑名单 |

### 5.4 群组接口

| 方法 | 参数 | 说明 |
|------|------|------|
| `CreateGroup` | callback, operationID, groupReqInfo | 创建群组 |
| `JoinGroup` | callback, operationID, groupID, reqMsg, joinSource, ex | 加入群组 |
| `QuitGroup` | callback, operationID, groupID | 退出群组 |
| `DismissGroup` | callback, operationID, groupID | 解散群组 |
| `SetGroupInfo` | callback, operationID, groupInfo | 设置群组信息 |
| `GetJoinedGroupList` | callback, operationID | 获取已加入群组列表 |
| `GetGroupMemberList` | callback, operationID, groupID, filter, offset, count | 获取群成员列表 |
| `TransferGroupOwner` | callback, operationID, groupID, newOwnerUserID | 转让群主 |
| `KickGroupMember` | callback, operationID, groupID, reason, userIDList | 踢出群成员 |

### 5.5 消息接口

| 方法 | 参数 | 说明 |
|------|------|------|
| `SendMessage` | callback, operationID, message, recvID, groupID, offlinePushInfo | 发送消息 |
| `GetAdvancedHistoryMessageList` | callback, operationID, getMessageOptions | 获取历史消息 |
| `RevokeMessage` | callback, operationID, conversationID, clientMsgID | 撤回消息 |
| `DeleteMessage` | callback, operationID, conversationID, clientMsgID | 删除消息 |
| `MarkConversationMessageAsRead` | callback, operationID, conversationID | 标记已读 |
| `SearchLocalMessages` | callback, operationID, searchParam | 搜索本地消息 |

### 5.6 会话接口

| 方法 | 参数 | 说明 |
|------|------|------|
| `GetAllConversationList` | callback, operationID | 获取所有会话 |
| `GetConversationListSplit` | callback, operationID, offset, count | 分页获取会话 |
| `GetOneConversation` | callback, operationID, sessionType, sourceID | 获取单个会话 |
| `SetConversation` | callback, operationID, conversationID, req | 设置会话 |
| `GetTotalUnreadMsgCount` | callback, operationID | 获取总未读数 |

### 5.7 第三方服务接口

| 方法 | 参数 | 说明 |
|------|------|------|
| `UploadFile` | callback, operationID, req, progress | 上传文件 |
| `UploadLogs` | callback, operationID, line, ex, progress | 上传日志 |
| `UpdateFcmToken` | callback, operationID, fcmToken, expireTime | 更新 FCM Token |
| `SetAppBadge` | callback, operationID, appUnreadCount | 设置应用角标 |
| `Logs` | callback, operationID, logLevel, file, line, msgs, err, keyAndValue | 客户端日志 |

### 5.8 在线状态接口

| 方法 | 参数 | 说明 |
|------|------|------|
| `SubscribeUsersStatus` | callback, operationID, userIDs | 订阅用户在线状态 |
| `UnsubscribeUsersStatus` | callback, operationID, userIDs | 取消订阅用户在线状态 |
| `GetSubscribeUsersStatus` | callback, operationID | 获取已订阅用户状态 |

---

## 6. 同步策略规范

### 6.1 版本同步机制

每个业务模块使用 `VersionSyncModel` 维护同步版本：

```go
type LocalVersionSync struct {
    TableName  string    // 表名
    EntityID   string    // 实体ID
    Version    string    // 版本号
    SyncTime   time.Time // 同步时间
}
```

### 6.2 同步流程

```
┌─────────────┐    ┌──────────────┐    ┌─────────────┐
│  本地数据库  │───▶│ 版本对比     │───▶│ 服务端API   │
│  (SQLite)   │    │ (VersionSync)│    │  (HTTP)     │
└─────────────┘    └──────────────┘    └─────────────┘
       │                    │                   │
       │◀───────────────────┼───────────────────│
       │         增量/全量同步结果               │
       ▼
┌─────────────┐
│  Syncer处理  │
│ Insert/     │
│ Update/     │
│ Delete/     │
│ Notice      │
└─────────────┘
```

### 6.3 同步限制

| 模块 | 同步限制 | 分页大小 |
|------|----------|----------|
| 好友 | 10,000 | 100 |
| 群组 | 1,000 | 100 |
| 群成员 | 1,000 | 100 |
| 会话 | MaxInt64 | 300 |

---

## 7. 错误码规范

### 7.1 SDK 错误码

| 错误码 | 说明 |
|--------|------|
| 10001 | 参数错误 |
| 10002 | SDK未初始化 |
| 10003 | 重复登录 |
| 10004 | 未登录 |
| 10005 | 网络超时 |
| 10006 | 网络错误 |
| 10007 | 用户ID不存在 |
| 10008 | 群组ID不存在 |

### 7.2 服务端错误码

| 错误码 | 说明 |
|--------|------|
| 13002 | 已在其他设备登录 |
| 1507 | Token过期 |
| 1601 | 已被拉黑 |
| 1602 | 不是好友 |
| 1701 | 不在群组中 |
| 1702 | 群组已解散 |

---

## 8. 平台适配规范

### 8.1 平台ID定义

| 平台 | ID |
|------|-----|
| iOS | 1 |
| Android | 2 |
| Windows | 3 |
| macOS | 4 |
| Web | 5 |
| Linux | 7 |
| AndroidPad | 8 |
| iPad | 9 |

### 8.2 构建目标

| 平台 | 构建方式 |
|------|----------|
| Android/iOS | gomobile |
| Web | WebAssembly (wasm) |
| Windows/macOS/Linux | CGO + 动态库 |

---

## 9. 性能规范

### 9.1 消息处理性能

- 消息批量处理: 100条/批
- 搜索协程限制: 10个并发
- 事件队列缓冲: 1000条

### 9.2 缓存策略

- 会话缓存: `Cache[string, *LocalConversation]`
- 群信息缓存: `Cache[string, *LocalGroup]`
- 群成员缓存: `Cache[string, *LocalGroupMember]`
- 用户信息缓存: `UserCache[string, *LocalUser]`

### 9.3 断线重连策略

- 最大重连次数: 300
- 重连策略: 指数退避
- 心跳间隔: 24秒
- Pong超时: 30秒

---

## 10. 安全规范

### 10.1 Token 管理

- Token 通过 Login 接口传入
- Token 过期自动触发登出
- 被踢下线通过 WebSocket 通知

### 10.2 数据加密

- WebSocket 连接支持压缩 (Gzip)
- 本地数据库使用 SQLite 文件存储
- 消息内容支持私密消息模式 (IsNotPrivate)

### 10.3 权限控制

- 群主/管理员权限检查
- 好友关系验证
- 黑名单消息过滤

---

## 11. 扩展规范

### 11.1 自定义消息

```go
type CustomElem struct {
    Data        string `json:"data"`        // 自定义数据
    Description string `json:"description"` // 描述
    Extension   string `json:"extension"`   // 扩展
}
```

### 11.2 消息扩展字段

```go
type AttachedInfoElem struct {
    GroupHasReadInfo  GroupHasReadInfo
    IsPrivateChat     bool
    BurnDuration      int32          // 阅后即焚时长
    HasReadTime       int64
    IsEncryption      bool
    Progress          *UploadProgress
}
```

### 11.3 业务回调

```go
type OnCustomBusinessListener interface {
    OnRecvCustomBusinessMessage(businessMessage string)
}
```

---

## 12. 测试规范

### 12.1 测试目录结构

```
test/           # 单元测试
integration_test/  # 集成测试
msgtest/        # 消息压力测试
```

### 12.2 测试配置

```json
{
    "APIADDR": "http://your-server-api-address",
    "WSADDR": "ws://your-server-websocket-address",
    "UserID": "your-test-user-id"
}
```

### 12.3 运行测试

```bash
go test -run TestFunctionName
```

---

## 13. 可执行 Task 清单

### Phase 1: 基础设施层 (P0 - 最高优先级)

| ID | Task | 描述 | 验收标准 | 预估复杂度 |
|----|------|------|----------|-----------|
| T1.1 | 初始化项目结构 | 确认 go.mod、目录结构、依赖版本 | `go mod tidy` 无错误 | 低 |
| T1.2 | 配置管理模块 | 实现 `IMConfig` 结构体解析与校验 | 能正确解析JSON配置，字段校验通过 | 低 |
| T1.3 | 日志系统 | 集成日志框架，支持级别/文件/轮转 | 日志按级别输出到文件和控制台 | 低 |
| T1.4 | SQLite 数据库初始化 | 实现 `InitDB` 方法，创建所有表 | 数据库文件创建成功，所有表存在 | 中 |
| T1.5 | 数据库接口实现 | 实现 `DataBase` 接口所有方法 | 所有CRUD操作通过单元测试 | 高 |

### Phase 2: 连接管理模块 (P0 - 核心)

| ID | Task | 描述 | 验收标准 | 预估复杂度 |
|----|------|------|----------|-----------|
| T2.1 | WebSocket 连接封装 | 实现 `LongConn` 接口，支持连接/断开 | 能建立和关闭WebSocket连接 | 中 |
| T2.2 | 消息编解码器 | 实现 Gob 编码 + Gzip 压缩 | 消息编码后能正确解码 | 中 |
| T2.3 | 心跳机制 | 实现 ping/pong 心跳保活 | 30s无pong自动重连 | 中 |
| T2.4 | 断线重连 | 实现指数退避重连策略 | 最多300次重连，间隔递增 | 中 |
| T2.5 | 消息发送队列 | 实现 send channel + writePump | 消息按序发送，不丢失 | 高 |
| T2.6 | 消息接收处理 | 实现 readPump + handleMessage | 收到消息正确分发到事件队列 | 高 |
| T2.7 | 请求-响应同步器 | 实现 `WsRespAsyn` 等待服务端响应 | SendReqWaitResp 超时正确处理 | 高 |
| T2.8 | 连接状态回调 | 触发 OnConnListener 各状态回调 | 连接/断开/被踢等事件正确回调 | 中 |

### Phase 3: 用户模块 (P1)

| ID | Task | 描述 | 验收标准 | 预估复杂度 |
|----|------|------|----------|-----------|
| T3.1 | 用户数据模型 | 实现 `LocalUser` 结构体与表映射 | 能正确存储和读取用户信息 | 低 |
| T3.2 | 用户 Syncer | 实现用户信息同步器 | 能从服务端同步用户信息到本地 | 中 |
| T3.3 | 用户缓存 | 实现 `UserCache` LRU缓存 | 缓存命中减少数据库查询 | 低 |
| T3.4 | GetUsersInfo API | 实现获取用户信息接口 | 返回正确的用户信息 | 中 |
| T3.5 | GetSelfUserInfo API | 实现获取自己信息接口 | 返回当前登录用户信息 | 低 |
| T3.6 | SetSelfInfo API | 实现设置自己信息接口 | 本地和远程信息都更新 | 中 |

### Phase 4: 关系链模块 (P1)

| ID | Task | 描述 | 验收标准 | 预估复杂度 |
|----|------|------|----------|-----------|
| T4.1 | 好友数据模型 | 实现 `LocalFriend` 结构体与表映射 | 能正确存储和读取好友信息 | 低 |
| T4.2 | 黑名单数据模型 | 实现 `LocalBlack` 结构体与表映射 | 能正确存储和读取黑名单信息 | 低 |
| T4.3 | 好友 Syncer | 实现好友列表同步器 | 分页同步，版本控制 | 高 |
| T4.4 | 黑名单 Syncer | 实现黑名单同步器 | 分页同步，版本控制 | 中 |
| T4.5 | GetFriendList API | 实现获取好友列表接口 | 支持分页和过滤黑名单 | 中 |
| T4.6 | AddFriend API | 实现添加好友接口 | 发送申请，触发回调 | 中 |
| T4.7 | DeleteFriend API | 实现删除好友接口 | 本地删除，远程通知 | 中 |
| T4.8 | Accept/RefuseFriend API | 实现接受/拒绝好友申请 | 状态更新，触发回调 | 中 |
| T4.9 | AddBlack/GetBlackList/RemoveBlack API | 实现黑名单管理接口 | 增删查操作正常 | 中 |
| T4.10 | 好友申请通知处理 | 处理好友申请推送消息 | 触发 OnFriendshipListener 回调 | 中 |

### Phase 5: 群组模块 (P1)

| ID | Task | 描述 | 验收标准 | 预估复杂度 |
|----|------|------|----------|-----------|
| T5.1 | 群组数据模型 | 实现 `LocalGroup` 结构体与表映射 | 能正确存储和读取群组信息 | 低 |
| T5.2 | 群成员数据模型 | 实现 `LocalGroupMember` 结构体与表映射 | 能正确存储和读取群成员信息 | 低 |
| T5.3 | 群组 Syncer | 实现群组列表同步器 | 分页同步，版本控制 | 高 |
| T5.4 | 群成员 Syncer | 实现群成员同步器 | 分页同步，版本控制 | 高 |
| T5.5 | 群组缓存 | 实现 `groupInfoCache` 和 `groupMemberCache` | 缓存命中减少查询 | 低 |
| T5.6 | CreateGroup API | 实现创建群组接口 | 创建成功，本地插入 | 中 |
| T5.7 | JoinGroup API | 实现加入群组接口 | 申请/直接加入，状态更新 | 中 |
| T5.8 | QuitGroup API | 实现退出群组接口 | 本地删除，远程通知 | 中 |
| T5.9 | DismissGroup API | 实现解散群组接口 | 仅群主可操作 | 中 |
| T5.10 | SetGroupInfo API | 实现设置群组信息接口 | 信息更新，触发回调 | 中 |
| T5.11 | GetJoinedGroupList API | 实现获取已加入群组列表 | 返回所有已加入群组 | 中 |
| T5.12 | GetGroupMemberList API | 实现获取群成员列表接口 | 支持分页和角色过滤 | 中 |
| T5.13 | TransferGroupOwner API | 实现转让群主接口 | 仅群主可操作 | 低 |
| T5.14 | KickGroupMember API | 实现踢出群成员接口 | 仅群主/管理员可操作 | 中 |
| T5.15 | 群组通知处理 | 处理群组变动推送消息 | 触发 OnGroupListener 回调 | 高 |

### Phase 6: 会话消息模块 (P0 - 核心)

| ID | Task | 描述 | 验收标准 | 预估复杂度 |
|----|------|------|----------|-----------|
| T6.1 | 会话数据模型 | 实现 `LocalConversation` 结构体与表映射 | 能正确存储和读取会话信息 | 中 |
| T6.2 | 消息数据模型 | 实现 `LocalChatLog` 结构体与表映射 | 支持所有消息类型存储 | 高 |
| T6.3 | 会话 Syncer | 实现会话列表同步器 | 分页同步，版本控制 | 高 |
| T6.4 | 消息发送器 | 实现 `messageSender` | 支持所有消息类型发送 | 高 |
| T6.5 | SendMessage API | 实现发送消息接口 | 消息发送成功，状态更新 | 高 |
| T6.6 | 消息创建方法 | 实现 CreateTextMessage 等所有创建方法 | 每种消息类型能正确创建 | 高 |
| T6.7 | GetAdvancedHistoryMessageList API | 实现获取历史消息接口 | 分页拉取，本地缓存 | 高 |
| T6.8 | RevokeMessage API | 实现撤回消息接口 | 消息标记撤回，通知对方 | 中 |
| T6.9 | DeleteMessage API | 实现删除消息接口 | 本地删除，不通知对方 | 低 |
| T6.10 | MarkConversationMessageAsRead API | 实现标记已读接口 | 发送已读回执，更新未读数 | 中 |
| T6.11 | SearchLocalMessages API | 实现搜索本地消息接口 | 支持关键词/类型/时间搜索 | 高 |
| T6.12 | 消息推送处理 | 处理 WebSocket 推送的新消息 | 正确插入本地数据库 | 高 |
| T6.13 | 消息同步器 | 实现 MsgSyncer 全量/增量同步 | 重装后全量，日常增量 | 高 |
| T6.14 | 未读数管理 | 实现未读数计算与更新 | 总未读数正确 | 中 |
| T6.15 | 正在输入状态 | 实现 typing 状态管理 | 发送/接收输入状态 | 低 |

### Phase 7: 第三方服务模块 (P2)

| ID | Task | 描述 | 验收标准 | 预估复杂度 |
|----|------|------|----------|-----------|
| T7.1 | 文件上传 | 实现图片/视频/文件上传 | 上传成功返回URL | 中 |
| T7.2 | 文件下载 | 实现文件下载与缓存 | 下载成功，本地缓存 | 中 |
| T7.3 | GetUsersInfoWithCache API | 带缓存的用户信息查询 | 缓存命中减少请求 | 低 |

### Phase 8: API 层与回调 (P1)

| ID | Task | 描述 | 验收标准 | 预估复杂度 |
|----|------|------|----------|-----------|
| T8.1 | InitSDK/UnInitSDK | 实现SDK初始化与反初始化 | 初始化成功，资源正确释放 | 中 |
| T8.2 | Login/Logout | 实现登录与登出 | 登录成功启动所有模块，登出清理资源 | 高 |
| T8.3 | GetLoginStatus | 实现获取登录状态 | 返回正确的登录状态 | 低 |
| T8.4 | 回调接口实现 | 实现所有 OnXxxListener 接口 | 各事件正确回调到上层 | 高 |
| T8.5 | 跨平台接口导出 | 导出 gomobile/wasm/CGO 接口 | 各平台能调用SDK方法 | 高 |

### Phase 9: 测试与质量保障 (P2)

| ID | Task | 描述 | 验收标准 | 预估复杂度 |
|----|------|------|----------|-----------|
| T9.1 | 单元测试 - 数据库层 | 覆盖所有数据库操作 | 覆盖率 > 80% | 高 |
| T9.2 | 单元测试 - 业务逻辑层 | 覆盖核心业务逻辑 | 覆盖率 > 70% | 高 |
| T9.3 | 集成测试 | 端到端消息收发测试 | 消息收发正常 | 高 |
| T9.4 | 压力测试 | 大量消息同步性能测试 | 10000条消息同步 < 10s | 中 |
| T9.5 | 内存泄漏检测 | 检测 goroutine 泄漏 | 无泄漏 | 中 |

### Task 依赖关系

```
Phase 1 (基础设施)
    ↓
Phase 2 (连接管理) ← 所有模块依赖
    ↓
┌─── Phase 3 (用户)
├─── Phase 4 (关系链)
├─── Phase 5 (群组)
└─── Phase 6 (会话消息) ← 核心业务
    ↓
Phase 7 (第三方服务)
    ↓
Phase 8 (API层与回调)
    ↓
Phase 9 (测试与质量保障)
```

---

## 附录

### A. 依赖清单

| 依赖 | 版本 | 用途 |
|------|------|------|
| gorm.io/gorm | v1.25.10 | ORM框架 |
| gorm.io/driver/sqlite | v1.5.5 | SQLite驱动 |
| github.com/gorilla/websocket | v1.4.2 | WebSocket客户端 |
| github.com/openimsdk/protocol | v0.0.73-alpha.12 | 协议定义 |
| github.com/openimsdk/tools | v0.0.50-alpha.80 | 工具库 |
| github.com/hashicorp/golang-lru/v2 | v2.0.7 | LRU缓存 |
| github.com/patrickmn/go-cache | v2.1.0 | 内存缓存 |

### B. 关键文件索引

| 文件 | 职责 |
|------|------|
| `open_im_sdk/init_login.go` | SDK初始化与登录 |
| `open_im_sdk/userRelated.go` | UserContext聚合根 |
| `open_im_sdk/conversation_msg.go` | 会话消息API |
| `open_im_sdk/group.go` | 群组API |
| `open_im_sdk/relation.go` | 关系链API |
| `open_im_sdk/user.go` | 用户API |
| `open_im_sdk/third.go` | 第三方服务API |
| `internal/interaction/long_conn_mgr.go` | 长连接管理 |
| `internal/conversation_msg/conversation_msg.go` | 会话消息逻辑 |
| `internal/group/group.go` | 群组逻辑 |
| `internal/relation/relation.go` | 关系链逻辑 |
| `internal/user/user.go` | 用户逻辑 |
| `pkg/db/db_interface/databse.go` | 数据库接口 |
| `open_im_sdk_callback/callback_client.go` | 回调接口定义 |
| `sdk_struct/sdk_struct.go` | SDK数据结构 |
| `pkg/constant/constant.go` | 常量定义 |

---

## 附录 C. 核心模块详细实现指南

> 本节提供核心模块的详细实现描述与代码片段，开发者可直接参考本文档进行开发，无需依赖本地项目文件。

### C.1 SDK 初始化与登录实现

#### 功能描述

SDK 初始化与登录是整个系统的入口点。`InitSDK` 函数接收 JSON 格式的配置字符串和连接监听器，解析配置后初始化日志系统，验证 API 和 WebSocket 地址格式，最后调用 `UserContext.InitSDK` 保存配置引用。

登录流程通过 `Login` 函数触发，实际执行在 `UserContext.login` 方法中完成。登录过程分为三个阶段：
1. **状态检查**: 防止重复登录
2. **资源初始化**: 创建数据库实例、初始化各业务模块、加载消息序列号
3. **启动运行**: 启动长连接管理器、消息同步器监听器、会话事件处理器

#### 关键实现代码

```go
// InitSDK - SDK 初始化入口
// 参数:
//   - listener: 连接状态监听器回调接口
//   - operationID: 操作追踪ID
//   - config: JSON格式的配置字符串
// 返回: bool 表示初始化是否成功
func InitSDK(listener open_im_sdk_callback.OnConnListener, operationID string, config string) bool {
    // 1. 解析 JSON 配置
    var configArgs sdk_struct.IMConfig
    if err := json.Unmarshal([]byte(config), &configArgs); err != nil {
        return false
    }
    
    // 2. 验证平台ID
    if configArgs.PlatformID == 0 {
        return false
    }
    
    // 3. 初始化日志系统
    log.InitLoggerFromConfig(
        "open-im-sdk-core", "", 
        configArgs.SystemType, 
        pbConstant.PlatformID2Name[int(configArgs.PlatformID)],
        int(configArgs.LogLevel), 
        configArgs.IsLogStandardOutput, 
        false, 
        configArgs.LogFilePath, 
        uint(logRemainCount), 
        rotationTime, 
        version.Version, 
        true,
    )
    
    // 4. 验证地址格式
    if !strings.Contains(configArgs.ApiAddr, "http") {
        return false // API 必须是 http/https 格式
    }
    if !strings.Contains(configArgs.WsAddr, "ws") {
        return false // WebSocket 必须是 ws/wss 格式
    }
    
    // 5. 调用 UserContext 初始化
    return IMUserContext.InitSDK(&configArgs, listener)
}

// UserContext.login - 登录核心逻辑
func (u *UserContext) login(ctx context.Context, userID, token string) error {
    // 1. 检查是否已登录
    if u.getLoginStatus(ctx) == Logged {
        return sdkerrs.ErrLoginRepeat
    }
    u.setLoginStatus(Logging)
    
    // 2. 保存用户信息
    u.info.UserID = userID
    u.info.Token = token
    
    // 3. 初始化资源（数据库、各模块）
    if err := u.initialize(ctx, userID); err != nil {
        return err
    }
    
    // 4. 启动所有服务
    u.run(ctx)
    u.setLoginStatus(Logged)
    
    return nil
}

// UserContext.initialize - 资源初始化
func (u *UserContext) initialize(ctx context.Context, userID string) error {
    // 1. 创建数据库实例
    u.db, err = db.NewDataBase(ctx, userID, u.info.DataDir, int(u.info.LogLevel))
    
    // 2. 检查并处理发送中的消息（将发送中状态改为失败）
    u.checkSendingMessage(ctx)
    
    // 3. 初始化各业务模块
    u.user.SetLoginUserID(userID)
    u.user.SetDataBase(u.db)
    u.relation.SetDataBase(u.db)
    u.group.SetDataBase(u.db)
    u.msgSyncer.SetDataBase(u.db)
    u.conversation.SetDataBase(u.db)
    
    // 4. 加载消息序列号（从数据库读取已同步的最大seq）
    err = u.msgSyncer.LoadSeq(ctx)
    
    return err
}

// UserContext.run - 启动所有服务
func (u *UserContext) run(ctx context.Context) {
    // 1. 启动长连接管理器（readPump, writePump, heartbeat）
    u.longConnMgr.Run(ctx, u.fgCtx)
    
    // 2. 启动消息同步器监听
    go u.msgSyncer.DoListener(ctx)
    
    // 3. 启动会话事件处理器
    go common.DoListener(u.ctx, u.conversation)
    
    // 4. 启动登出监听器
    go u.logoutListener(ctx)
}
```

#### 登录状态机

```
LogoutStatus (1) ──Login──▶ Logging (2) ──成功──▶ Logged (3)
                                │                      │
                                │ 失败                 │ Logout
                                ▼                      ▼
                           LogoutStatus (1) ◀──── LogoutStatus (1)
```

---

### C.2 WebSocket 长连接管理实现

#### 功能描述

`LongConnMgr` 是 SDK 的核心组件，负责管理 WebSocket 连接的完整生命周期。它采用三个独立 goroutine 分别处理消息读取、消息写入和心跳保活。

连接管理器实现了以下关键功能：
- **智能重连**: 指数退避策略，最多重试 300 次
- **消息压缩**: 使用 Gzip 压缩传输数据
- **消息编解码**: 使用 Gob 编码
- **有序消息发送**: 支持文本和媒体两个独立通道，保证消息顺序
- **心跳保活**: 每 24 秒发送一次 ping，30 秒超时
- **前后台切换**: 支持应用进入后台时暂停部分 goroutine

#### 连接状态机

```
DefaultNotConnect (0) ──▶ Connecting (2) ──▶ Connected (4)
                              │                    │
                              │ 失败               │ 断线
                              ▼                    ▼
                         等待重连 ◀──── Reconnecting (最多300次)
                                                 │
                                                 │ 主动关闭
                                                 ▼
                                            Closed (1)
```

#### 关键实现代码

```go
// LongConnMgr 结构体定义
type LongConnMgr struct {
    w          sync.Mutex          // 连接状态互斥锁
    connStatus int                 // 连接状态
    conn       LongConn            // WebSocket 连接实例
    listener   func() OnConnListener // 监听器
    send       chan Message        // 发送队列（缓冲大小10）
    Syncer     *WsRespAsyn         // 异步响应同步器
    encoder    Encoder             // 消息编码器 (Gob)
    compressor Compressor          // 消息压缩器 (Gzip)
    connWrite  *sync.Mutex         // 写入锁
    sub        *subscription       // 用户在线状态订阅
    mb         *MessageBatcher     // 消息批处理器
}

// 时间常量定义
const (
    writeWait          = 10 * time.Second  // 写入超时
    pongWait           = 30 * time.Second  // 等待pong超时
    pingPeriod         = (pongWait * 8) / 10 // 24秒，发送ping间隔
    maxMessageSize     = 1024 * 1024       // 最大消息大小 1MB
    maxReconnectAttempts = 300             // 最大重连次数
    sendAndWaitTime    = time.Second * 10  // 发送等待响应超时
    sendChainMaxWait   = 3 * time.Second   // 发送链最大等待时间
)

// Run - 启动三个核心 goroutine
func (c *LongConnMgr) Run(ctx, fgCtx context.Context) {
    go c.readPump(ctx, fgCtx)    // 读取消息
    go c.writePump(ctx)          // 写入消息
    go c.heartbeat(ctx, fgCtx)   // 心跳保活
}

// readPump - 消息读取循环
func (c *LongConnMgr) readPump(ctx context.Context, fgCtx context.Context) {
    defer func() {
        if r := recover(); r != nil {
            // panic 恢复
            log.ZWarn(ctx, "readPump panic", nil, "panic info", fmt.Sprintf("%+v\n%s", r, debug.Stack()))
        }
    }()
    
    connNum := 0
    for {
        select {
        case <-ctx.Done():
            return // SDK 登出
        case <-fgCtx.Done():
            return // 应用进入后台
        default:
        }
        
        // 1. 尝试重连
        needRecon, err := c.reConn(ctx, &connNum)
        if !needRecon {
            return // 不需要重连，退出
        }
        if err != nil {
            time.Sleep(c.reconnectStrategy.GetSleepInterval())
            continue
        }
        
        // 2. 读取消息
        c.conn.SetReadLimit(maxMessageSize)
        _ = c.conn.SetReadDeadline(pongWait)
        messageType, message, err := c.conn.ReadMessage()
        if err != nil {
            _ = c.close()
            continue
        }
        
        // 3. 处理消息
        switch messageType {
        case MessageBinary:
            c.handleMessage(message) // 处理二进制消息
        case MessageText:
            return // 不支持文本消息
        case CloseMessage:
            return // 连接关闭
        }
    }
}

// writePump - 消息写入循环
func (c *LongConnMgr) writePump(ctx context.Context) {
    defer func() {
        if r := recover(); r != nil {
            log.ZWarn(ctx, "writePump panic", nil, "panic info", fmt.Sprintf("%+v\n%s", r, debug.Stack()))
        }
    }()
    
    // 创建两个通道状态（文本和媒体）
    textLane := newLaneState(ccontext.SendOrderLaneText)
    mediaLane := newLaneState(ccontext.SendOrderLaneMedia)
    
    for {
        select {
        case <-ctx.Done():
            return
        case message, ok := <-c.send:
            if !ok {
                // 发送通道关闭
                _ = c.conn.WriteMessage(websocket.CloseMessage, []byte{})
                return
            }
            // 处理 incoming 消息
            c.processIncomingMessage(textLane, mediaLane, message)
        case <-textTimer:
            c.handleLaneTimeout(textLane) // 处理通道超时
        case <-mediaTimer:
            c.handleLaneTimeout(mediaLane)
        }
    }
}

// heartbeat - 心跳保活
func (c *LongConnMgr) heartbeat(ctx context.Context, fgCtx context.Context) {
    ticker := time.NewTicker(pingPeriod)
    defer ticker.Stop()
    
    for {
        select {
        case <-ctx.Done():
            return
        case <-fgCtx.Done():
            return
        case <-ticker.C:
            c.sendPingMessage(ctx) // 发送 ping 消息
        }
    }
}

// handleMessage - 处理接收到的消息
func (c *LongConnMgr) handleMessage(message []byte) error {
    // 1. 解压缩
    if c.IsCompression {
        message, decompressErr = c.compressor.DecompressWithPool(message)
        if decompressErr != nil {
            return sdkerrs.ErrMsgDeCompression
        }
    }
    
    // 2. 解码
    var wsResp GeneralWsResp
    err := c.encoder.Decode(message, &wsResp)
    if err != nil {
        return sdkerrs.ErrMsgDecodeBinaryWs
    }
    
    // 3. 根据消息类型分发处理
    switch wsResp.ReqIdentifier {
    case constant.PushMsg:
        c.doPushMsg(ctx, wsResp) // 处理推送消息
    case constant.LogoutMsg:
        return sdkerrs.ErrLoginOut // 登出通知
    case constant.KickOnlineMsg:
        return errs.ErrTokenKicked // 踢下线通知
    case constant.GetNewestSeq, constant.PullMsgByRange, constant.SendMsg:
        c.Syncer.NotifyResp(ctx, wsResp) // 通知响应
    }
    return nil
}

// SendReqWaitResp - 发送请求并等待响应（同步调用）
func (c *LongConnMgr) SendReqWaitResp(ctx context.Context, m proto.Message, reqIdentifier int, resp proto.Message) error {
    // 1. 序列化请求
    data, err := proto.Marshal(m)
    
    // 2. 创建消息并放入发送队列
    msg := Message{
        Message: GeneralWsReq{
            ReqIdentifier: reqIdentifier,
            SendID:        ccontext.Info(ctx).UserID(),
            OperationID:   ccontext.Info(ctx).OperationID(),
            Data:          data,
        },
        Resp: make(chan *GeneralWsResp, 1),
    }
    c.send <- msg
    
    // 3. 等待响应
    select {
    case <-ctx.Done():
        return sdkerrs.ErrCtxDeadline
    case v, ok := <-msg.Resp:
        if !ok || v.ErrCode != 0 {
            return errs.NewCodeError(v.ErrCode, v.ErrMsg)
        }
        // 4. 反序列化响应
        return proto.Unmarshal(v.Data, resp)
    }
}
```

#### 消息协议标识

| 标识常量 | 值 | 用途 | 处理逻辑 |
|---------|-----|------|---------|
| `GetNewestSeq` | 1001 | 获取最新序列号 | 同步器等待响应 |
| `PullMsgByRange` | 1002 | 拉取消息范围 | 同步器等待响应 |
| `SendMsg` | 1003 | 发送消息 | 同步器等待响应 |
| `PushMsg` | 2001 | 推送消息 | 分发到会话事件队列 |
| `KickOnlineMsg` | 2002 | 踢下线通知 | 触发踢下线回调 |
| `LogoutMsg` | 2003 | 登出通知 | 触发登出流程 |

---

### C.3 消息同步机制实现

#### 功能描述

`MsgSyncer` 是消息同步的核心组件，负责管理消息的全量同步和增量同步。它通过维护每个会话的最大已同步序列号（`syncedMaxSeqs`）来判断需要同步的消息范围。

同步策略分为两种场景：
1. **重新安装同步**: 应用首次安装或重装后，执行全量同步，拉取所有历史消息
2. **增量同步**: 日常使用中，只拉取缺失的消息

#### 同步流程图

```
登录成功
    │
    ▼
LoadSeq() - 从数据库加载已同步的最大seq
    │
    ▼
doConnected() - 连接成功后触发同步
    │
    ├── 发送 GetMaxSeqReq 获取服务端最大seq
    │
    ▼
compareSeqsAndBatchSync() - 比较seq差异
    │
    ├── 重新安装场景:
    │   ├── 通知类会话: 直接保存seq到数据库
    │   └── 普通会话: 计算需要拉取的seq范围
    │
    └── 日常场景:
        └── 计算需要拉取的seq范围
            │
            ▼
syncAndTriggerMsgs() - 分批同步消息
    │
    ├── 按 100 条消息分批拉取
    │
    ▼
pullMsgBySeqRange() - 从服务端拉取消息
    │
    ▼
triggerConversation() - 触发会话处理
    │
    ▼
更新 syncedMaxSeqs
```

#### 关键实现代码

```go
// MsgSyncer 结构体定义
type MsgSyncer struct {
    loginUserID            string                // 登录用户ID
    longConnMgr            *LongConnMgr          // 长连接管理器
    PushMsgAndMaxSeqCh     chan common.Cmd2Value // 推送消息和最大seq通道
    conversationEventQueue chan common.Cmd2Value // 会话事件队列
    syncedMaxSeqs          map[string]int64      // 已同步的最大seq映射表
    syncedMaxSeqsLock      sync.RWMutex          // seq映射表锁
    db                     db_interface.DataBase // 数据库接口
    reinstalled            bool                  // 是否重新安装
    isSyncing              bool                  // 是否正在同步
    isSyncingLock          sync.Mutex            // 同步状态锁
}

// 同步常量
const (
    connectPullNums       = 1     // 连接时拉取数量
    defaultPullNums       = 10    // 默认拉取数量
    SplitPullMsgNum       = 100   // 分批拉取消息数
    pullMsgGoroutineLimit = 10    // 拉取goroutine限制
    maxConversations      = 500   // 最大会话数
    synMaxConversations   = 100   // 同步最大会话数
)

// LoadSeq - 从数据库加载已同步的序列号
func (m *MsgSyncer) LoadSeq(ctx context.Context) error {
    // 1. 获取所有会话ID
    conversationIDList, err := m.db.GetAllConversationIDList(ctx)
    
    // 2. 判断是否重新安装
    if len(conversationIDList) == 0 {
        version, err := m.db.GetAppSDKVersion(ctx)
        if version == nil || !version.Installed {
            m.reinstalled = true // 标记为重新安装
        }
    }
    
    // 3. 并发加载每个会话的最大seq
    partSize := 20
    currency := (len(conversationIDList)-1)/partSize + 1
    var wg sync.WaitGroup
    resultMaps := make([]map[string]SyncedSeq, currency)
    
    for i := 0; i < currency; i++ {
        wg.Add(1)
        go func(i, start, end int) {
            defer wg.Done()
            for _, v := range conversationIDList[start:end] {
                maxSyncedSeq, err := m.db.CheckConversationNormalMsgSeq(ctx, v)
                resultMaps[i][v] = SyncedSeq{
                    ConversationID: v,
                    MaxSyncedSeq:   maxSyncedSeq,
                    Err:            err,
                }
            }
        }(i, i*partSize, min(i*partSize+partSize, len(conversationIDList)))
    }
    wg.Wait()
    
    // 4. 合并结果到 syncedMaxSeqs
    for _, resultMap := range resultMaps {
        for k, v := range resultMap {
            if v.Err == nil {
                m.syncedMaxSeqs[k] = v.MaxSyncedSeq
            }
        }
    }
    
    // 5. 加载通知类会话seq
    notificationSeqs, err := m.db.GetNotificationAllSeqs(ctx)
    for _, notificationSeq := range notificationSeqs {
        m.syncedMaxSeqs[notificationSeq.ConversationID] = notificationSeq.Seq
    }
    
    return nil
}

// DoListener - 监听消息同步事件
func (m *MsgSyncer) DoListener(ctx context.Context) {
    for {
        select {
        case cmd := <-m.PushMsgAndMaxSeqCh:
            m.handlePushMsgAndEvent(cmd)
        case <-ctx.Done():
            return
        }
    }
}

// handlePushMsgAndEvent - 处理推送消息和事件
func (m *MsgSyncer) handlePushMsgAndEvent(cmd common.Cmd2Value) {
    switch cmd.Cmd {
    case constant.CmdConnSuccesss:
        // 连接成功，触发同步
        if m.startSync() {
            m.doConnected(cmd.Ctx)
        }
    case constant.CmdWakeUpDataSync:
        // 应用唤醒，数据同步
        if m.startSync() {
            m.doWakeupDataSync(cmd.Ctx)
        }
    case constant.CmdIMMessageSync:
        // 手动触发IM消息同步
        if conversationIDs, ok := cmd.Value.([]string); ok {
            m.doIMMessageSync(cmd.Ctx, conversationIDs)
        }
    case constant.CmdPushMsg:
        // 处理推送消息
        m.doPushMsg(cmd.Ctx, cmd.Value.(*sdkws.PushMessages))
    }
}

// doConnected - 连接成功后的同步
func (m *MsgSyncer) doConnected(ctx context.Context) {
    reinstalled := m.reinstalled
    
    // 1. 发送同步开始标志
    if reinstalled {
        common.DispatchSyncFlag(ctx, constant.AppDataSyncStart, m.conversationEventQueue)
    } else {
        common.DispatchSyncFlag(ctx, constant.MsgSyncBegin, m.conversationEventQueue)
    }
    
    // 2. 获取服务端最大seq（最多重试3次）
    var resp sdkws.GetMaxSeqResp
    err := m.longConnMgr.SendReqWaitResp(ctx, 
        &sdkws.GetMaxSeqReq{UserID: m.loginUserID}, 
        constant.GetNewestSeq, &resp)
    if err != nil {
        common.DispatchSyncFlag(ctx, constant.MsgSyncFailed, m.conversationEventQueue)
        return
    }
    
    // 3. 比较并同步
    m.compareSeqsAndBatchSync(ctx, resp.MaxSeqs, connectPullNums)
    
    // 4. 发送同步完成标志
    if reinstalled {
        common.DispatchSyncFlag(ctx, constant.AppDataSyncFinish, m.conversationEventQueue)
    } else {
        common.DispatchSyncFlag(ctx, constant.MsgSyncEnd, m.conversationEventQueue)
    }
}

// compareSeqsAndBatchSync - 比较seq差异并批量同步
func (m *MsgSyncer) compareSeqsAndBatchSync(ctx context.Context, maxSeqToSync map[string]int64, pullNums int64) {
    needSyncSeqMap := make(map[string][2]int64)
    
    // 计算每个会话需要同步的seq范围
    for conversationID, maxSeq := range maxSeqToSync {
        if syncedMaxSeq, ok := m.syncedMaxSeqs[conversationID]; ok {
            if maxSeq > syncedMaxSeq {
                // [已同步seq+1, 最大seq]
                needSyncSeqMap[conversationID] = [2]int64{syncedMaxSeq + 1, maxSeq}
            }
        } else {
            if maxSeq != 0 {
                // 新会话，从0开始同步
                needSyncSeqMap[conversationID] = [2]int64{0, maxSeq}
            }
        }
    }
    
    // 根据场景选择同步方法
    if m.reinstalled {
        m.syncAndTriggerReinstallMsgs(ctx, needSyncSeqMap, pullNums)
    } else {
        m.syncAndTriggerMsgs(ctx, needSyncSeqMap, pullNums)
    }
}

// syncAndTriggerMsgs - 分批同步消息
func (m *MsgSyncer) syncAndTriggerMsgs(ctx context.Context, seqMap map[string][2]int64, syncMsgNum int64) error {
    if len(seqMap) == 0 {
        return nil
    }
    
    var tempSeqMap = make(map[string][2]int64, 50)
    var msgNum = 0
    
    for k, v := range seqMap {
        oneConversationSyncNum := v[1] - v[0] + 1
        tempSeqMap[k] = v
        msgNum += int(min(oneConversationSyncNum, syncMsgNum))
        
        // 达到批处理大小时触发拉取
        if msgNum >= SplitPullMsgNum {
            resp, err := m.pullMsgBySeqRange(ctx, tempSeqMap, syncMsgNum)
            if err != nil {
                return err
            }
            _ = m.triggerConversation(ctx, resp.Msgs)
            _ = m.triggerNotification(ctx, resp.NotificationMsgs)
            
            // 更新已同步的seq
            for conversationID, seqs := range tempSeqMap {
                m.syncedMaxSeqs[conversationID] = seqs[1]
            }
            tempSeqMap = make(map[string][2]int64, 50)
            msgNum = 0
        }
    }
    
    // 处理剩余消息
    if len(tempSeqMap) > 0 {
        resp, err := m.pullMsgBySeqRange(ctx, tempSeqMap, syncMsgNum)
        if err != nil {
            return err
        }
        _ = m.triggerConversation(ctx, resp.Msgs)
        _ = m.triggerNotification(ctx, resp.NotificationMsgs)
        for conversationID, seqs := range tempSeqMap {
            m.syncedMaxSeqs[conversationID] = seqs[1]
        }
    }
    
    return nil
}

// doPushMsg - 处理推送消息
func (m *MsgSyncer) doPushMsg(ctx context.Context, push *sdkws.PushMessages) {
    // 处理普通消息
    m.pushTriggerAndSync(ctx, push.Msgs, m.triggerConversation)
    // 处理通知消息
    m.pushTriggerAndSync(ctx, push.NotificationMsgs, m.triggerNotification)
}

// pushTriggerAndSync - 推送触发并同步
func (m *MsgSyncer) pushTriggerAndSync(ctx context.Context, pushMessages map[string]*sdkws.PullMsgs,
    triggerFunc func(ctx context.Context, msgs map[string]*sdkws.PullMsgs) error) {
    
    needSyncSeqMap := make(map[string][2]int64)
    res := make(map[string]*sdkws.PullMsgs)
    
    for conversationID, msgs := range pushMessages {
        var lastSeq int64
        var storageMsgs []*sdkws.MsgData
        
        for _, msg := range msgs.Msgs {
            if msg.Seq == 0 {
                // seq为0的消息直接触发（通常是通知类消息）
                _ = triggerFunc(ctx, map[string]*sdkws.PullMsgs{
                    conversationID: {Msgs: []*sdkws.MsgData{msg}},
                })
                continue
            }
            lastSeq = msg.Seq
            storageMsgs = append(storageMsgs, msg)
        }
        
        // 检查消息是否连续
        expectedLast := m.syncedMaxSeqs[conversationID] + int64(len(storageMsgs))
        if lastSeq == expectedLast {
            // 消息连续，直接触发
            res[conversationID] = &sdkws.PullMsgs{Msgs: storageMsgs}
            m.syncedMaxSeqs[conversationID] = lastSeq
        } else if lastSeq > m.syncedMaxSeqs[conversationID] {
            // 消息不连续，需要同步缺失的消息
            needSyncSeqMap[conversationID] = [2]int64{
                m.syncedMaxSeqs[conversationID] + 1,
                lastSeq,
            }
        }
    }
    
    if len(res) > 0 {
        _ = triggerFunc(ctx, res)
    }
    m.syncAndTriggerMsgs(ctx, needSyncSeqMap, defaultPullNums)
}

// startSync - 检查是否正在同步，防止并发同步
func (m *MsgSyncer) startSync() bool {
    m.isSyncingLock.Lock()
    defer m.isSyncingLock.Unlock()
    
    if m.isSyncing {
        return false
    }
    
    m.isSyncing = true
    // 5秒后自动释放同步状态
    go func() {
        time.Sleep(5 * time.Second)
        m.isSyncingLock.Lock()
        m.isSyncing = false
        m.isSyncingLock.Unlock()
    }()
    
    return true
}
```

---

### C.4 数据库模块实现

#### 功能描述

数据库模块基于 SQLite 和 GORM 实现，为每个会话动态创建独立的消息表。数据库接口采用接口隔离原则，将不同业务模块的数据操作分离到不同的 Model 接口中。

核心设计特点：
- **动态表创建**: 每个会话根据 conversationID 动态创建消息表
- **读写锁控制**: 所有数据库操作使用读写锁保护
- **索引优化**: 为 seq 和 send_time 字段创建索引
- **表存在性检查**: 操作前检查表是否存在，不存在则自动创建

#### 数据库结构

```
SQLite 数据库
├── local_conversations (会话表)
├── local_friends (好友表)
├── local_blacks (黑名单表)
├── local_groups (群组表)
├── local_group_members (群成员表)
├── local_users (用户表)
├── local_version_sync (版本同步表)
├── local_app_sdk_version (SDK版本表)
├── notification_seqs (通知序列号表)
├── sending_messages (发送中消息表)
└── {conversationID} (动态消息表，每个会话一个)
```

#### 关键实现代码

```go
// DataBase 接口定义
type DataBase interface {
    Close(ctx context.Context) error
    InitDB(ctx context.Context, userID string, dataDir string) error
    GroupModel
    MessageModel
    ConversationModel
    UserModel
    FriendModel
    S3Model
    SendingMessagesModel
    VersionSyncModel
    AppSDKVersion
    TableMaster
}

// 消息表结构
type LocalChatLog struct {
    ClientMsgID      string `gorm:"column:client_msg_id;primary_key;type:char(64)"`
    ServerMsgID      string `gorm:"column:server_msg_id;type:char(64)"`
    SendID           string `gorm:"column:send_id;type:char(64)"`
    RecvID           string `gorm:"column:recv_id;type:char(64)"`
    SenderPlatformID int32  `gorm:"column:sender_platform_id"`
    SenderNickname   string `gorm:"column:sender_nick_name;type:varchar(255)"`
    SenderFaceURL    string `gorm:"column:sender_face_url;type:varchar(255)"`
    SessionType      int32  `gorm:"column:session_type"`
    MsgFrom          int32  `gorm:"column:msg_from"`
    ContentType      int32  `gorm:"column:content_type"`
    Content          string `gorm:"column:content;type:varchar(1000)"`
    IsRead           bool   `gorm:"column:is_read"`
    Status           int32  `gorm:"column:status"`
    Seq              int64  `gorm:"column:seq;default:0"`
    SendTime         int64  `gorm:"column:send_time"`
    CreateTime       int64  `gorm:"column:create_time"`
    AttachedInfo     string `gorm:"column:attached_info;type:varchar(1024)"`
    Ex               string `gorm:"column:ex;type:varchar(1024)"`
    LocalEx          string `gorm:"column:local_ex;type:varchar(1024)"`
}

// initChatLog - 动态创建消息表
func (d *DataBase) initChatLog(ctx context.Context, conversationID string) error {
    d.mRWMutex.Lock()
    defer d.mRWMutex.Unlock()
    
    tableName := utils.GetTableName(conversationID)
    if !d.tableChecker.HasTable(tableName) {
        // 1. 创建表
        createTableSQL := fmt.Sprintf(`
            CREATE TABLE "%s" (
                client_msg_id CHAR(64),
                server_msg_id CHAR(64),
                send_id CHAR(64),
                recv_id CHAR(64),
                sender_platform_id INTEGER,
                sender_nick_name VARCHAR(255),
                sender_face_url VARCHAR(255),
                session_type INTEGER,
                msg_from INTEGER,
                content_type INTEGER,
                content VARCHAR(1000),
                is_read NUMERIC,
                status INTEGER,
                seq INTEGER DEFAULT 0,
                send_time INTEGER,
                create_time INTEGER,
                attached_info VARCHAR(1024),
                ex VARCHAR(1024),
                local_ex VARCHAR(1024),
                is_react NUMERIC,
                is_external_extensions NUMERIC,
                msg_first_modify_time INTEGER,
                PRIMARY KEY (client_msg_id)
            );`, tableName)
        
        if result := d.conn.Exec(createTableSQL); result.Error != nil {
            return errs.WrapMsg(result.Error, "Create table failed", "table", tableName)
        }
        
        // 2. 创建索引
        d.conn.Exec(fmt.Sprintf("CREATE INDEX `%s` ON `%s` (seq)", 
            "index_seq_"+conversationID, tableName))
        d.conn.Exec(fmt.Sprintf("CREATE INDEX `%s` ON `%s` (send_time)", 
            "index_send_time_"+conversationID, tableName))
        
        d.tableChecker.UpdateTable(tableName)
    }
    return nil
}

// InsertMessage - 插入单条消息
func (d *DataBase) InsertMessage(ctx context.Context, conversationID string, Message *model_struct.LocalChatLog) error {
    d.mRWMutex.Lock()
    defer d.mRWMutex.Unlock()
    return errs.WrapMsg(
        d.conn.WithContext(ctx).Table(utils.GetTableName(conversationID)).Create(Message).Error, 
        "InsertMessage failed",
    )
}

// BatchInsertMessageList - 批量插入消息
func (d *DataBase) BatchInsertMessageList(ctx context.Context, conversationID string, MessageList []*model_struct.LocalChatLog) error {
    // 1. 确保表存在
    err := d.initChatLog(ctx, conversationID)
    if err != nil {
        return err
    }
    
    if MessageList == nil {
        return nil
    }
    
    d.mRWMutex.Lock()
    defer d.mRWMutex.Unlock()
    return errs.WrapMsg(
        d.conn.WithContext(ctx).Table(utils.GetTableName(conversationID)).Create(MessageList).Error, 
        "BatchInsertMessageList failed",
    )
}

// GetMessageList - 分页获取消息列表
func (d *DataBase) GetMessageList(ctx context.Context, conversationID string, count int, startTime, startSeq int64, startClientMsgID string, isReverse bool) (result []*model_struct.LocalChatLog, err error) {
    if err = d.initChatLog(ctx, conversationID); err != nil {
        return nil, err
    }
    
    d.mRWMutex.RLock()
    defer d.mRWMutex.RUnlock()
    
    var condition, timeOrder, timeSymbol string
    if isReverse {
        timeOrder = "send_time ASC,seq ASC"
        timeSymbol = ">"
    } else {
        timeOrder = "send_time DESC,seq DESC"
        timeSymbol = "<"
    }
    
    if startTime > 0 {
        // 带时间戳的分页查询
        condition = "send_time " + timeSymbol + " ? " +
            "OR (send_time = ? AND (seq " + timeSymbol + " ? OR (seq = 0 AND client_msg_id != ?)))"
        err = errs.WrapMsg(
            d.conn.WithContext(ctx).Table(utils.GetTableName(conversationID)).
                Where(condition, startTime, startTime, startSeq, startClientMsgID).
                Order(timeOrder).Offset(0).Limit(count).Find(&result).Error, 
            "GetMessageList failed",
        )
    } else {
        // 不带时间戳的分页查询
        err = errs.WrapMsg(
            d.conn.WithContext(ctx).Table(utils.GetTableName(conversationID)).
                Order(timeOrder).Offset(0).Limit(count).Find(&result).Error, 
            "GetMessageList failed",
        )
    }
    
    return result, err
}

// UpdateMessage - 更新消息
func (d *DataBase) UpdateMessage(ctx context.Context, conversationID string, c *model_struct.LocalChatLog) error {
    d.mRWMutex.Lock()
    defer d.mRWMutex.Unlock()
    t := d.conn.WithContext(ctx).Table(utils.GetTableName(conversationID)).Updates(c)
    if t.RowsAffected == 0 {
        return errs.WrapMsg(errors.New("RowsAffected == 0"), "no update")
    }
    return errs.WrapMsg(t.Error, "UpdateMessage failed")
}

// UpdateMessageTimeAndStatus - 更新消息状态和时间
func (d *DataBase) UpdateMessageTimeAndStatus(ctx context.Context, conversationID, clientMsgID string, serverMsgID string, sendTime int64, status int32) error {
    d.mRWMutex.Lock()
    defer d.mRWMutex.Unlock()
    return errs.WrapMsg(
        d.conn.WithContext(ctx).Table(utils.GetTableName(conversationID)).
            Model(model_struct.LocalChatLog{}).
            Where("client_msg_id=? And seq=?", clientMsgID, 0).
            Updates(model_struct.LocalChatLog{
                Status: status, 
                SendTime: sendTime, 
                ServerMsgID: serverMsgID,
            }).Error, 
        "UpdateMessageStatusBySourceID failed",
    )
}

// MarkConversationMessageAsReadDB - 标记消息已读
func (d *DataBase) MarkConversationMessageAsReadDB(ctx context.Context, conversationID string, msgIDs []string) (rowsAffected int64, err error) {
    d.mRWMutex.Lock()
    defer d.mRWMutex.Unlock()
    
    var msgs []*model_struct.LocalChatLog
    if err := d.conn.WithContext(ctx).Table(utils.GetConversationTableName(conversationID)).
        Where("client_msg_id in ? AND send_id != ?", msgIDs, d.loginUserID).
        Find(&msgs).Error; err != nil {
        return 0, errs.WrapMsg(err, "MarkConversationMessageAsReadDB failed")
    }
    
    for _, msg := range msgs {
        var attachedInfo sdk_struct.AttachedInfoElem
        utils.JsonStringToStruct(msg.AttachedInfo, &attachedInfo)
        attachedInfo.HasReadTime = utils.GetCurrentTimestampByMill()
        msg.IsRead = true
        msg.AttachedInfo = utils.StructToJsonString(attachedInfo)
        
        if err := d.conn.WithContext(ctx).Table(utils.GetConversationTableName(conversationID)).
            Where("client_msg_id = ?", msg.ClientMsgID).Updates(msg).Error; err != nil {
            log.ZError(ctx, "MarkConversationMessageAsReadDB failed", err, "msg", msg)
        } else {
            rowsAffected++
        }
    }
    
    return rowsAffected, nil
}

// SearchMessageByContentType - 按内容类型搜索消息
func (d *DataBase) SearchMessageByContentType(ctx context.Context, contentType []int, senderUserIDList []string, conversationID string, startTime, endTime int64, offset, count int) (result []*model_struct.LocalChatLog, err error) {
    d.mRWMutex.RLock()
    defer d.mRWMutex.RUnlock()
    
    query := d.conn.WithContext(ctx).Table(utils.GetTableName(conversationID)).
        Where("send_time BETWEEN ? AND ?", startTime, endTime).
        Where("status <= ?", constant.MsgStatusSendFailed).
        Where("content_type IN ?", contentType)
    
    if len(senderUserIDList) != 0 {
        query = query.Where("send_id IN ?", senderUserIDList)
    }
    
    err = errs.WrapMsg(
        query.Order("send_time DESC").Offset(offset).Limit(count).Find(&result).Error, 
        "SearchMessage failed",
    )
    
    return result, err
}

// SearchMessageByKeyword - 按关键词搜索消息
func (d *DataBase) SearchMessageByKeyword(ctx context.Context, contentType []int, senderUserIDList []string, keywordList []string, keywordListMatchType int, conversationID string, startTime, endTime int64, offset, count int) (result []*model_struct.LocalChatLog, err error) {
    d.mRWMutex.RLock()
    defer d.mRWMutex.RUnlock()
    
    query := d.conn.WithContext(ctx).Table(utils.GetTableName(conversationID)).
        Where("send_time BETWEEN ? AND ?", startTime, endTime).
        Where("status <= ?", constant.MsgStatusSendFailed).
        Where("content_type IN ?", contentType)
    
    // 关键词匹配逻辑
    if len(keywordList) > 0 {
        if keywordListMatchType == constant.KeywordMatchOr {
            // OR 逻辑: 匹配任意关键词
            orConditions := make([]string, len(keywordList))
            args := make([]any, len(keywordList))
            for i, keyword := range keywordList {
                orConditions[i] = "content LIKE ?"
                args[i] = "%" + keyword + "%"
            }
            query = query.Where("("+strings.Join(orConditions, " OR ")+")", args...)
        } else {
            // AND 逻辑: 匹配所有关键词
            for _, keyword := range keywordList {
                query = query.Where("content LIKE ?", "%"+keyword+"%")
            }
        }
    }
    
    if len(senderUserIDList) != 0 {
        query = query.Where("send_id IN ?", senderUserIDList)
    }
    
    err = errs.WrapMsg(
        query.Order("send_time DESC").Offset(offset).Limit(count).Find(&result).Error, 
        "SearchMessage failed",
    )
    
    return result, err
}
```

---

### C.5 会话消息处理实现

#### 功能描述

`Conversation` 模块是 SDK 的核心业务模块，负责会话管理、消息处理、消息同步触发等功能。它通过事件队列接收来自消息同步器的消息，然后进行解析、存储和回调通知。

消息处理流程：
1. 从事件队列接收消息
2. 按会话分组处理
3. 区分自己发送和他人发送的消息
4. 检查消息是否已存在（去重）
5. 插入或更新数据库
6. 更新会话列表
7. 触发回调通知

#### 关键实现代码

```go
// Conversation 结构体定义
type Conversation struct {
    *interaction.LongConnMgr
    conversationSyncer          *syncer.Syncer[*model_struct.LocalConversation, ...]
    db                          db_interface.DataBase
    ConversationListener        func() open_im_sdk_callback.OnConversationListener
    msgListener                 func() open_im_sdk_callback.OnAdvancedMsgListener
    msgSyncerCh                 chan common.Cmd2Value
    conversationEventQueue      chan common.Cmd2Value
    loginUserID                 string
    platform                    int32
    relation                    *relation.Relation
    group                       *group.Group
    user                        *user.User
    file                        *file.File
    cache                       *cache.Cache[string, *model_struct.LocalConversation]
    maxSeqRecorder              MaxSeqRecorder
    sender                      *messageSender
    typing                      *typing
}

// doMsgNew - 处理新消息（核心方法）
func (c *Conversation) doMsgNew(c2v common.Cmd2Value) {
    allMsg := c2v.Value.(sdk_struct.CmdNewMsgComeToConversation).Msgs
    ctx := c2v.Ctx
    
    // 1. 初始化存储映射
    insertMsg := make(map[string][]*model_struct.LocalChatLog, 10)
    updateMsg := make(map[string][]*model_struct.LocalChatLog, 10)
    var newMessages sdk_struct.NewMsgList
    conversationChangedSet := make(map[string]*model_struct.LocalConversation)
    newConversationSet := make(map[string]*model_struct.LocalConversation)
    
    // 2. 遍历每个会话的消息
    for conversationID, msgs := range allMsg {
        // 获取本地已存在的消息（用于去重）
        clientIDs := make([]string, 0, len(msgs.Msgs))
        for _, msg := range msgs.Msgs {
            clientIDs = append(clientIDs, msg.ClientMsgID)
        }
        clientMsgs, _ := c.db.GetMessagesByClientMsgIDs(ctx, conversationID, clientIDs)
        clientMsgMap := datautil.SliceToMap(clientMsgs, func(e *model_struct.LocalChatLog) string {
            return e.ClientMsgID
        })
        
        var insertMessage, selfInsertMessage, othersInsertMessage []*model_struct.LocalChatLog
        var updateMessage []*model_struct.LocalChatLog
        
        // 3. 遍历每条消息
        for _, v := range msgs.Msgs {
            msg := converter.MsgDataToMsgStruct(v)
            
            // 消息已被云端标记删除，直接插入本地
            if msg.Status == constant.MsgStatusHasDeleted {
                dbMessage := converter.MsgStructToLocalChatLog(msg)
                insertMessage = append(insertMessage, dbMessage)
                continue
            }
            
            msg.Status = constant.MsgStatusSendSuccess
            
            // 解析消息内容
            err := converter.PopulateMsgStructByContentType(msg)
            if err != nil {
                continue
            }
            
            if v.SendID == c.loginUserID {
                // 自己发送的消息
                existingMsg, ok := clientMsgMap[msg.ClientMsgID]
                if ok {
                    if existingMsg.Seq == 0 {
                        // 消息正在发送中，更新状态
                        updateMessage = append(updateMessage, converter.MsgStructToLocalChatLog(msg))
                    } else {
                        // 消息已存在，作为异常消息处理
                        insertMessage = append(insertMessage, converter.MsgStructToLocalChatLog(msg))
                    }
                } else {
                    // 同步的消息，插入数据库
                    selfInsertMessage = append(selfInsertMessage, converter.MsgStructToLocalChatLog(msg))
                }
            } else {
                // 他人发送的消息
                existingMsg, ok := clientMsgMap[msg.ClientMsgID]
                if !ok {
                    // 新消息，构建会话信息
                    lc := model_struct.LocalConversation{
                        ConversationType:  v.SessionType,
                        LatestMsg:         utils.StructToJsonString(msg),
                        LatestMsgSendTime: msg.SendTime,
                        ConversationID:    conversationID,
                    }
                    switch v.SessionType {
                    case constant.SingleChatType:
                        lc.UserID = v.SendID
                        lc.ShowName = msg.SenderNickname
                        lc.FaceURL = msg.SenderFaceURL
                    case constant.WriteGroupChatType, constant.ReadGroupChatType:
                        lc.GroupID = v.GroupID
                    }
                    
                    // 更新未读数
                    if c.maxSeqRecorder.IsNewMsg(conversationID, msg.Seq) {
                        lc.UnreadCount = 1
                        c.maxSeqRecorder.Incr(conversationID, 1)
                    }
                    
                    // 添加到新消息列表
                    newMessages = append(newMessages, msg)
                    othersInsertMessage = append(othersInsertMessage, converter.MsgStructToLocalChatLog(msg))
                }
            }
        }
        
        insertMsg[conversationID] = append(insertMessage, 
            c.faceURLAndNicknameHandle(ctx, selfInsertMessage, othersInsertMessage, conversationID)...)
        if len(updateMessage) > 0 {
            updateMsg[conversationID] = updateMessage
        }
    }
    
    // 4. 锁定会话同步互斥锁
    c.conversationSyncMutex.Lock()
    defer c.conversationSyncMutex.Unlock()
    
    // 5. 获取本地会话
    list, _ := c.db.GetMultipleConversationDB(ctx, conversationIDs)
    m := datautil.SliceToMap(list, func(e *model_struct.LocalConversation) string {
        return e.ConversationID
    })
    
    // 6. 比较本地和远程会话，确定新增和变更
    c.diff(ctx, m, conversationSet, conversationChangedSet, newConversationSet)
    
    // 7. 批量更新和插入消息
    _ = c.batchUpdateMessageList(ctx, updateMsg)
    _ = c.batchInsertMessageList(ctx, insertMsg)
    
    // 8. 更新会话列表
    c.db.BatchUpdateConversationList(ctx, append(
        datautil.Values(conversationChangedSet), 
        datautil.Values(phConversationChangedSet)...,
    ))
    c.db.BatchInsertConversationList(ctx, datautil.Values(phNewConversationSet))
    
    // 9. 触发新消息回调
    c.newMessage(ctx, newMessages, conversationChangedSet, newConversationSet, onlineMap)
    
    // 10. 触发会话变更回调
    if len(newConversationSet) > 0 {
        c.ConversationListener().OnNewConversation(utils.StructToJsonString(datautil.Values(newConversationSet)))
    }
    if len(conversationChangedSet) > 0 {
        c.ConversationListener().OnConversationChanged(utils.StructToJsonString(datautil.Values(conversationChangedSet)))
    }
    
    // 11. 触发总未读数变更回调
    if isTriggerUnReadCount {
        _ = c.OnTotalUnreadMessageCountChanged(ctx)
    }
}
```

---

### C.6 Syncer 数据同步框架实现

#### 功能描述

`Syncer` 是一个泛型数据同步框架，为各业务模块（会话、好友、群组等）提供统一的同步能力。它支持全量同步、增量同步、版本控制、批量分页请求等功能。

Syncer 生命周期：
- **Insert**: 新增数据时触发
- **Delete**: 删除数据时触发
- **Update**: 更新数据时触发
- **Notice**: 状态变更通知

#### 关键实现代码

```go
// Syncer 泛型同步器定义
type Syncer[T any, Resp any, Key any] struct {
    insertFunc    func(ctx context.Context, value T) error
    deleteFunc    func(ctx context.Context, value T) error
    updateFunc    func(ctx context.Context, server, local T) error
    noticeFunc    func(ctx context.Context, state int, server, local T) error
    uuidFunc      func(value T) Key
    equalFunc     func(server, local T) bool
    batchPageReq  func(entityID Key) page.PageReq
    pageRespFunc  func(resp *Resp) []T
    apiRouter     string
    fullSyncLimit int64
}

// 创建 Syncer 实例（使用函数选项模式）
func New2[T any, Resp any, Key any](opts ...Option[T, Resp, Key]) *Syncer[T, Resp, Key] {
    s := &Syncer[T, Resp, Key]{}
    for _, opt := range opts {
        opt(s)
    }
    return s
}

// 函数选项
func WithInsert[T any, Resp any, Key any](f func(ctx context.Context, value T) error) Option[T, Resp, Key] {
    return func(s *Syncer[T, Resp, Key]) {
        s.insertFunc = f
    }
}

func WithUpdate[T any, Resp any, Key any](f func(ctx context.Context, server, local T) error) Option[T, Resp, Key] {
    return func(s *Syncer[T, Resp, Key]) {
        s.updateFunc = f
    }
}

func WithEqual[T any, Resp any, Key any](f func(server, local T) bool) Option[T, Resp, Key] {
    return func(s *Syncer[T, Resp, Key]) {
        s.equalFunc = f
    }
}

func WithBatchPageReq[T any, Resp any, Key any](f func(entityID Key) page.PageReq) Option[T, Resp, Key] {
    return func(s *Syncer[T, Resp, Key]) {
        s.batchPageReq = f
    }
}

// 会话 Syncer 初始化示例
func (c *Conversation) initSyncer() {
    c.conversationSyncer = syncer.New2[*model_struct.LocalConversation, pbConversation.GetOwnerConversationResp, string](
        // 插入回调
        syncer.WithInsert(func(ctx context.Context, value *model_struct.LocalConversation) error {
            c.batchAddFaceURLAndName(ctx, value)
            return c.db.InsertConversation(ctx, value)
        }),
        // 删除回调
        syncer.WithDelete(func(ctx context.Context, value *model_struct.LocalConversation) error {
            return c.db.DeleteConversation(ctx, value.ConversationID)
        }),
        // 更新回调
        syncer.WithUpdate(func(ctx context.Context, server, local *model_struct.LocalConversation) error {
            return c.db.UpdateColumnsConversation(ctx, server.ConversationID, map[string]interface{}{
                "recv_msg_opt": server.RecvMsgOpt,
                "is_pinned": server.IsPinned,
                "is_private_chat": server.IsPrivateChat,
                // ... 更多字段
            })
        }),
        // UUID 函数
        syncer.WithUUID(func(value *model_struct.LocalConversation) string {
            return value.ConversationID
        }),
        // 相等比较
        syncer.WithEqual(func(server, local *model_struct.LocalConversation) bool {
            return server.RecvMsgOpt == local.RecvMsgOpt &&
                server.IsPinned == local.IsPinned &&
                server.MaxSeq == local.MaxSeq
                // ... 更多字段比较
        }),
        // 状态变更通知
        syncer.WithNotice(func(ctx context.Context, state int, server, local *model_struct.LocalConversation) error {
            if state == syncer.Update || state == syncer.Insert {
                c.doUpdateConversation(common.Cmd2Value{
                    Value: common.UpdateConNode{
                        ConID: server.ConversationID,
                        Action: constant.ConChange,
                        Args: []string{server.ConversationID},
                    },
                })
            }
            return nil
        }),
        // 批量分页请求
        syncer.WithBatchPageReq(func(entityID string) page.PageReq {
            return &pbConversation.GetOwnerConversationReq{
                UserID: entityID,
                Pagination: &sdkws.RequestPagination{ShowNumber: 300},
            }
        }),
        // API 路由
        syncer.WithReqApiRouter(api.GetOwnerConversation.Route()),
        // 全量同步限制
        syncer.WithFullSyncLimit(conversationSyncLimit),
    )
}
```

### C.7 第三方服务模块实现

#### 功能描述

`Third` 模块负责处理文件上传、日志上传、FCM Token 管理和应用角标设置等第三方服务相关功能。文件上传支持分片上传和断点续传，日志上传用于问题排查，FCM Token 用于移动端推送。

#### 关键实现代码

```go
// Third - 第三方服务管理器
type Third struct {
    platform      int32
    loginUserID   string
    appFramework  string
    LogFilePath   string
    fileUploader  *file.File
    logUploadLock sync.Mutex
}

// UploadFile - 文件上传入口
// 参数:
//   - req: JSON格式的文件上传请求（包含文件路径、名称、类型等）
//   - progress: 上传进度回调
// 返回: 上传结果（包含文件URL、文件大小等）
func (t *Third) UploadFile(ctx context.Context, req string, progress file.UploadFileCallback) (*structpb.Value, error) {
    var uploadFileReq file.UploadFileReq
    if err := json.Unmarshal([]byte(req), &uploadFileReq); err != nil {
        return nil, err
    }
    
    // 设置平台信息
    uploadFileReq.Platform = t.platform
    
    // 调用文件上传模块
    resp, err := t.fileUploader.UploadFile(ctx, &uploadFileReq, progress)
    if err != nil {
        return nil, err
    }
    
    return utils.StructPbMarshal(resp), nil
}

// UploadLogs - 日志上传
// 用于问题排查，将本地日志上传到服务器
func (t *Third) UploadLogs(ctx context.Context, line int, ex string, progress UploadLogProgress) error {
    t.logUploadLock.Lock()
    defer t.logUploadLock.Unlock()
    
    // 压缩日志文件
    zipPath, err := t.zipLogs(ctx)
    if err != nil {
        return err
    }
    
    // 上传压缩后的日志
    resp, err := t.fileUploader.UploadFile(ctx, &file.UploadFileReq{
        Name:     "logs.zip",
        FilePath: zipPath,
        Type:     "application/zip",
    }, &uploadLogProgress{progress: progress})
    
    return err
}

// UpdateFcmToken - 更新 FCM Token
// 用于 Firebase Cloud Messaging 推送
func (t *Third) UpdateFcmToken(ctx context.Context, fcmToken string, expireTime int64) error {
    if fcmToken == "" {
        return nil
    }
    
    // 调用服务端 API 更新 Token
    return t.updateFcmToken(ctx, fcmToken, expireTime)
}

// SetAppBadge - 设置应用角标
// 用于显示应用未读消息数
func (t *Third) SetAppBadge(ctx context.Context, appUnreadCount int32) error {
    // 调用服务端 API 设置角标
    return t.setAppBadge(ctx, appUnreadCount)
}
```

#### 文件上传流程

```
1. 计算文件 MD5
   └── 打开文件
   └── 读取文件内容
   └── 计算 MD5 哈希
   └── 回调 HashProgress

2. 检查文件是否已存在（秒传）
   └── 调用 InitiateMultipartUpload
   └── 如果文件已存在，直接返回 URL

3. 初始化分片上传
   └── 调用 InitiateMultipartUpload
   └── 获取 UploadID

4. 分片上传
   └── 按大小分割文件（默认 10MB/片）
   └── 并发上传各分片
   └── 回调 PartComplete

5. 完成上传
   └── 调用 CompleteMultipartUpload
   └── 回调 Complete
   └── 返回文件 URL
```

### C.8 在线状态模块实现

#### 功能描述

在线状态模块用于订阅和获取用户的在线状态信息。通过 WebSocket 连接与服务端保持状态同步，支持多平台在线状态查询。

#### 关键实现代码

```go
// subscribeUsersStatus - 订阅用户在线状态（内部方法）
// 参数:
//   - userIDs: 要订阅的用户ID列表
// 返回: 用户在线状态列表
func (c *LongConnMgr) subscribeUsersStatus(ctx context.Context, userIDs []string) ([]*userPb.OnlineStatus, error) {
    if len(userIDs) == 0 {
        return []*userPb.OnlineStatus{}, nil
    }
    
    // 获取用户在线平台ID列表
    res, err := c.GetUserOnlinePlatformIDs(ctx, userIDs)
    if err != nil {
        return nil, err
    }
    
    // 构建在线状态列表
    status := make([]*userPb.OnlineStatus, 0, len(res))
    for userID, platformIDs := range res {
        value := &userPb.OnlineStatus{
            UserID:      userID,
            PlatformIDs: platformIDs,
        }
        if len(platformIDs) == 0 {
            value.Status = constant.Offline  // 离线
        } else {
            value.Status = constant.Online   // 在线
        }
        status = append(status, value)
    }
    return status, nil
}

// SubscribeUsersStatus - 订阅用户在线状态（公开方法）
func (c *LongConnMgr) SubscribeUsersStatus(ctx context.Context, userIDs []string) ([]*userPb.OnlineStatus, error) {
    if len(userIDs) == 0 {
        return []*userPb.OnlineStatus{}, nil
    }
    return c.subscribeUsersStatus(ctx, userIDs)
}

// UnsubscribeUsersStatus - 取消订阅用户在线状态
func (c *LongConnMgr) UnsubscribeUsersStatus(ctx context.Context, userIDs []string) error {
    return c.UnsubscribeUserOnlinePlatformIDs(ctx, userIDs)
}

// GetSubscribeUsersStatus - 获取已订阅用户的在线状态
func (c *LongConnMgr) GetSubscribeUsersStatus(ctx context.Context) ([]*userPb.OnlineStatus, error) {
    return c.subscribeUsersStatus(ctx, nil)  // nil 表示获取所有已订阅用户
}
```

#### 在线状态常量

```go
const (
    Offline = 0  // 离线
    Online  = 1  // 在线
)

// 平台ID定义
const (
    IOS     = 1  // iOS
    Android = 2  // Android
    Windows = 3  // Windows
    MacOS   = 4  // macOS
    Web     = 5  // Web
    Linux   = 7  // Linux
)
```

---

## 附录 D. 功能完整性验证

> 本节验证 SDD 文档是否完整覆盖原项目的所有功能，开发者可据此确认文档的可靠性。

### D.1 openim-sdk-core 功能覆盖清单

| 模块 | 原项目文件 | SDD 覆盖章节 | 状态 |
|------|-----------|-------------|------|
| **SDK 初始化** | `open_im_sdk/init_login.go` | C.1, 5.1 | ✅ |
| **UserContext 聚合根** | `open_im_sdk/userRelated.go` | 2.2.1, C.1 | ✅ |
| **长连接管理** | `internal/interaction/long_conn_mgr.go` | 3.1, C.2 | ✅ |
| **消息同步** | `internal/interaction/msg_sync.go` | 3.2.2, C.3 | ✅ |
| **会话管理** | `internal/conversation_msg/conversation_msg.go` | 3.2, C.5 | ✅ |
| **消息发送** | `internal/conversation_msg/send_queue.go` | 3.2, C.5 | ✅ |
| **消息创建** | `internal/conversation_msg/create_message.go` | 3.2, 5.5 | ✅ |
| **消息撤回** | `internal/conversation_msg/revoke.go` | 3.2, 5.5 | ✅ |
| **消息删除** | `internal/conversation_msg/delete.go` | 3.2, 5.5 | ✅ |
| **已读回执** | `internal/conversation_msg/read_drawing.go` | 3.2, 5.5 | ✅ |
| **历史消息** | `internal/conversation_msg/sync.go` | 3.2, C.5 | ✅ |
| **增量同步** | `internal/conversation_msg/incremental_sync.go` | 3.2.2, C.3 | ✅ |
| **群组管理** | `internal/group/group.go` | 3.3, 5.4 | ✅ |
| **群组同步** | `internal/group/full_sync.go` | 3.3, C.6 | ✅ |
| **群组增量同步** | `internal/group/incremental_sync.go` | 3.3, C.6 | ✅ |
| **好友管理** | `internal/relation/relation.go` | 3.4, 5.3 | ✅ |
| **好友同步** | `internal/relation/sync.go` | 3.4, C.6 | ✅ |
| **用户管理** | `internal/user/user.go` | 3.5, 5.2 | ✅ |
| **用户同步** | `internal/user/full_sync.go` | 3.5, C.6 | ✅ |
| **数据库接口** | `pkg/db/db_interface/databse.go` | 3.8, C.4 | ✅ |
| **消息存储** | `pkg/db/chat_log_model.go` | 3.8, C.4 | ✅ |
| **会话存储** | `pkg/db/conversation_model.go` | 3.8, C.4 | ✅ |
| **好友存储** | `pkg/db/friend_model.go` | 3.8, C.4 | ✅ |
| **群组存储** | `pkg/db/group_model.go` | 3.8, C.4 | ✅ |
| **数据库初始化** | `pkg/db/db_init.go` | 3.8, C.4 | ✅ |
| **Syncer 框架** | `pkg/syncer/syncer.go` | 2.2.2, C.6 | ✅ |
| **文件上传** | `internal/third/file/file.go` | 3.6, C.7 | ✅ |
| **日志上传** | `internal/third/third.go` | 3.6, C.7 | ✅ |
| **FCM Token** | `open_im_sdk/third.go` | 3.6, 5.7 | ✅ |
| **在线状态** | `internal/interaction/online.go` | 3.7, C.8 | ✅ |
| **回调接口** | `open_im_sdk_callback/callback_client.go` | 3.9 | ✅ |
| **数据结构** | `sdk_struct/sdk_struct.go` | 4.1, 4.2 | ✅ |
| **常量定义** | `pkg/constant/constant.go` | 3.2, 8.1 | ✅ |
| **错误码** | `pkg/sdkerrs/code.go` | 7.1 | ✅ |
| **API 封装** | `pkg/api/api.go` | 5 | ✅ |
| **缓存** | `pkg/cache/cache.go` | 9.2 | ✅ |
| **WASM 支持** | `wasm/cmd/main.go` | 8.2 | ✅ |
| **gomobile 支持** | `open_im_sdk/*.go` | 8.2 | ✅ |

### D.2 openim-flutter-demo 功能覆盖清单

| 模块 | 原项目文件 | SDD 覆盖章节 | 状态 |
|------|-----------|-------------|------|
| **应用入口** | `lib/main.dart` | 3.2.8, 15 | ✅ |
| **IM 控制器** | `lib/core/controller/im_controller.dart` | 3.1.1, D.1 | ✅ |
| **应用控制器** | `lib/core/controller/app_controller.dart` | 3.1.2, D.7 | ✅ |
| **IM 回调** | `lib/core/im_callback.dart` | 6 | ✅ |
| **启动页** | `lib/pages/splash/splash_logic.dart` | 3.2.8 | ✅ |
| **登录页** | `lib/pages/login/login_logic.dart` | 3.2.9 | ✅ |
| **首页** | `lib/pages/home/home_logic.dart` | 3.2.10 | ✅ |
| **会话列表** | `lib/pages/conversation/conversation_logic.dart` | 3.2.4, D.3 | ✅ |
| **聊天页** | `lib/pages/chat/chat_logic.dart` | 3.2.5, D.2 | ✅ |
| **聊天设置** | `lib/pages/chat/chat_setup/chat_setup_logic.dart` | 3.2, 15 | ✅ |
| **群设置** | `lib/pages/chat/group_setup/group_setup_logic.dart` | 3.2, 15 | ✅ |
| **通讯录** | `lib/pages/contacts/contacts_logic.dart` | 3.2.6 | ✅ |
| **好友列表** | `lib/pages/contacts/friend_list/friend_list_logic.dart` | 3.2, 15 | ✅ |
| **群列表** | `lib/pages/contacts/group_list/group_list_logic.dart` | 3.2, 15 | ✅ |
| **好友申请** | `lib/pages/contacts/friend_requests/friend_requests_logic.dart` | 3.2, 15 | ✅ |
| **群申请** | `lib/pages/contacts/group_requests/group_requests_logic.dart` | 3.2, 15 | ✅ |
| **用户资料** | `lib/pages/contacts/user_profile_panel/user_profile_panel_logic.dart` | 3.2, 15 | ✅ |
| **群资料** | `lib/pages/contacts/group_profile_panel/group_profile_panel_logic.dart` | 3.2, 15 | ✅ |
| **选择联系人** | `lib/pages/contacts/select_contacts/select_contacts_logic.dart` | 3.2, 15 | ✅ |
| **创建群组** | `lib/pages/contacts/create_group/create_group_logic.dart` | 3.2, 15 | ✅ |
| **我的** | `lib/pages/mine/mine_logic.dart` | 3.2.7 | ✅ |
| **个人信息** | `lib/pages/mine/my_info/my_info_logic.dart` | 3.2, 15 | ✅ |
| **编辑信息** | `lib/pages/mine/edit_my_info/edit_my_info_logic.dart` | 3.2, 15 | ✅ |
| **黑名单** | `lib/pages/mine/blacklist/blacklist_logic.dart` | 3.2, 15 | ✅ |
| **账号设置** | `lib/pages/mine/account_setup/account_setup_logic.dart` | 3.2, 15 | ✅ |
| **关于我们** | `lib/pages/mine/about_us/about_us_logic.dart` | 3.2, 15 | ✅ |
| **语言设置** | `lib/pages/mine/language_setup/language_setup_logic.dart` | 3.2, 15 | ✅ |
| **发现页** | `lib/pages/discover/discover_logic.dart` | 3.2, 15 | ✅ |
| **全局搜索** | `lib/pages/global_search/global_search_logic.dart` | 3.2, 15 | ✅ |
| **注册** | `lib/pages/register/register_logic.dart` | 3.2, 15 | ✅ |
| **忘记密码** | `lib/pages/forget_password/forget_password_logic.dart` | 3.2, 15 | ✅ |
| **路由** | `lib/routes/app_pages.dart` | 3.3, D.4 | ✅ |
| **导航器** | `lib/routes/app_navigator.dart` | 3.3, D.4 | ✅ |
| **升级管理** | `lib/utils/upgrade_manager.dart` | 3.2, 15 | ✅ |

### D.3 核心流程验证

| 流程 | 原项目实现 | SDD 描述 | 状态 |
|------|-----------|---------|------|
| SDK 初始化流程 | `InitSDK` → `UserContext.InitSDK` | C.1 | ✅ |
| 登录流程 | `Login` → `UserContext.login` → `initialize` → `run` | C.1 | ✅ |
| WebSocket 连接流程 | `Run` → `readPump` + `writePump` + `heartbeat` | C.2 | ✅ |
| 消息发送流程 | `SendMessage` → `messageSender` → `send` channel → `writePump` | C.5 | ✅ |
| 消息接收流程 | `readPump` → `handleMessage` → `doPushMsg` → `conversationEventQueue` | C.2, C.5 | ✅ |
| 消息同步流程 | `LoadSeq` → `compareSeqsAndBatchSync` → `syncAndTriggerMsgs` | C.3 | ✅ |
| 断线重连流程 | `readPump` → `reConn` → 指数退避 | C.2 | ✅ |
| Flutter 登录流程 | `SplashLogic` → `IMController.login` → `HomePage` | 3.2.8, 3.2.9 | ✅ |
| Flutter 消息发送 | `ChatLogic.sendTextMsg` → `createTextMessage` → `_sendMessage` | D.2 | ✅ |
| Flutter 消息接收 | `IMController.onRecvNewMessage` → `ChatLogic.messageList.add` | D.2 | ✅ |
| Flutter 会话列表 | `ConversationLogic` → `conversationAddedSubject` → 排序 | D.3 | ✅ |

### D.4 未覆盖功能说明

以下功能在原项目中存在，但在 SDD 文档中描述较为简略：

| 功能 | 原因 | 建议 |
|------|------|------|
| WASM 详细实现 | 平台特定，非核心业务 | 参考 `wasm/` 目录源码 |
| gomobile 导出 | 平台特定，非核心业务 | 参考 `docs/gomobile-android-ios-setup.md` |
| 集成测试框架 | 测试专用，非运行时功能 | 参考 `integration_test/` 目录 |
| 音视频通话详细实现 | 依赖外部模块 `openim_live` | 参考 `openim_live` 模块文档 |
| 图片压缩算法 | 依赖 `openim_common` 工具库 | 参考 `openim_common` 源码 |
