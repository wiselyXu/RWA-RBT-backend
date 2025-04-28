use std::{fs::File, io::Read};

use once_cell::sync::Lazy;
use structopt::StructOpt;

use super::cfgs::{Configs, Opt};

//  只要是配置文件中的配置项，都可以通过这个结构体来获取，
// 只要读取一次值后保存到内存，一直可供使用
pub static CFG: Lazy<Configs> = Lazy::new(self::Configs::init);

impl Configs {
    pub fn init() -> Self {
        let opt = Opt::from_args();
        println!("参数信息： {:?}", opt);
        let mut cfg_file = "config/config.toml".to_string();
        if opt.env == "prod".to_string() {
            cfg_file = "config/config_docker.toml".to_string();
        }

        let mut file = match File::open(cfg_file.clone()) {
            Ok(f) => f,
            Err(e) => panic!("不存在配置文件：{}，错误信息：{}", cfg_file.clone(), e),
        };
        let mut cfg_contents = String::new();
        match file.read_to_string(&mut cfg_contents) {
            Ok(s) => s,
            Err(e) => panic!("读取配置文件失败，错误信息：{}", e),
        };
        toml::from_str(&cfg_contents).expect("解析配置文件错误")
    }
}
