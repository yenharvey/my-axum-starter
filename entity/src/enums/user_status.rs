use num_enum::{IntoPrimitive, TryFromPrimitive};
use schemars::JsonSchema;
use sea_orm::{DeriveActiveEnum, EnumIter};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

/// 用户状态
///
/// 使用 i16 存储在数据库中，提高查询效率
#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    JsonSchema,
    DeriveActiveEnum,
    EnumIter,
    Display,
    EnumString,
    IntoPrimitive,
    TryFromPrimitive,
    PartialEq,
    Eq,
    Default,
)]
#[sea_orm(rs_type = "i16", db_type = "SmallInteger")]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
#[repr(i16)]
pub enum UserStatus {
    /// 激活状态
    #[default]
    Active = 0,

    /// 停用状态
    Inactive = 1,

    /// 删除状态
    Deleted = 2,
}
