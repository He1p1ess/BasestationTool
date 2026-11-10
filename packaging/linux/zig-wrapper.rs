use std::{env,path::Path,process::{Command,exit}};
fn main(){
 let exe=env::current_exe().unwrap(); let mut cmd=Command::new(exe.parent().unwrap().join("ziglang/zig.exe"));
 if exe.file_stem().unwrap()=="linux-ar"{cmd.arg("ar");}else{cmd.args(["cc","-target","x86_64-linux-gnu.2.31"]);}
 let base=env::current_dir().unwrap();
 for arg in env::args_os().skip(1){let path=Path::new(&arg);if let Ok(rel)=path.strip_prefix(&base){cmd.arg(rel);}else{cmd.arg(arg);}}
 exit(cmd.status().unwrap().code().unwrap_or(1));
}
