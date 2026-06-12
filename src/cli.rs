use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "bank_query", about = "金融机构查询系统")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 启动 Web 服务器（默认）
    Server,
    /// 初始化数据库
    InitDb,
    /// 设置管理员密码
    SetPassword {
        /// 新密码
        password: String,
    },
    /// 从 Excel 文件导入数据
    Import {
        /// Excel 文件路径
        file: String,
    },
    /// 清空所有数据
    Clear,
}
