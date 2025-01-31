use serde::{Deserialize, Serialize};
use serde_json::{self, json};
use std::sync::LazyLock;
pub mod info;
pub mod queryMobilePhone;
pub mod receive;

// 接口返回数据结构
#[derive(Serialize, Deserialize)]
pub struct ApiResponse {
    responseCode: String,
    success: bool,
    responseMsg: String,
}

// 优惠券id
pub static redPackTaskIdVec: LazyLock<serde_json::Value> = LazyLock::new(|| {
    json!(
        {
            // 三国杀
            "sgs": [57153, 57152, 57143, 58238, 58237, 57164, 57165, 59623],
            // 王者荣耀
            "wzry": [56639, 56640, 56665, 58233, 58234, 58231, 56653, 56656]
        }
    )
});
