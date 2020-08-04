pub mod server;

use crate::ConnectionId;
use actix::prelude::*;
use diesel::{
  r2d2::{ConnectionManager, Pool},
  PgConnection,
};
use log::{error, info};
use rand::{rngs::ThreadRng, Rng};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use server::ChatServer;
use std::{
  collections::{HashMap, HashSet},
  str::FromStr,
};

#[derive(EnumString, ToString, Debug, Clone)]
pub enum UserOperation {
  Login,
  Register,
  CreateCommunity,
  CreatePost,
  ListCommunities,
  ListCategories,
  GetPost,
  GetCommunity,
  GetCommunitySettings,
  CreateComment,
  EditComment,
  SaveComment,
  CreateCommentLike,
  CreateCommentReport,
  ListCommentReports,
  ResolveCommentReport,
  GetPosts,
  CreatePostLike,
  CreatePostReport,
  ListPostReports,
  ResolvePostReport,
  EditPost,
  SavePost,
  EditCommunity,
  EditCommunitySettings,
  FollowCommunity,
  GetFollowedCommunities,
  GetUserDetails,
  GetReplies,
  GetUserMentions,
  EditUserMention,
  GetModlog,
  BanFromCommunity,
  AddModToCommunity,
  CreateSite,
  EditSite,
  GetSite,
  AddAdmin,
  AddSitemod,
  BanUser,
  Search,
  MarkAllAsRead,
  SaveUserSettings,
  TransferCommunity,
  TransferSite,
  DeleteAccount,
  PasswordReset,
  PasswordChange,
  CreatePrivateMessage,
  EditPrivateMessage,
  GetPrivateMessages,
  UserJoin,
  GetComments,
  GetSiteConfig,
  SaveSiteConfig,
  GetReportCount,
  GetSiteModerators,
  GetUserTag,
  SetUserTag,
}

#[derive(Clone)]
pub struct WebsocketInfo {
  pub chatserver: Addr<ChatServer>,
  pub id: Option<ConnectionId>,
}
