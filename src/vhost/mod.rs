pub mod hosts;
pub mod manager;

pub use hosts::sync_hosts_file;
pub use manager::{
    add_virtual_host, detect_doc_root, load_vhosts, remove_virtual_host, save_vhosts,
    suggest_name_and_domain, VirtualHost,
};
