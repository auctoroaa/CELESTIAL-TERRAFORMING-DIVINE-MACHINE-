use celestial_terraforming_divine_machine::{genesis,GenesisIntent};
use std::{env,fs,process::ExitCode};
fn main()->ExitCode{
 let args:Vec<String>=env::args().collect();
 let intent=if let Some(path)=args.get(1){match fs::read_to_string(path).and_then(|s|serde_json::from_str::<GenesisIntent>(&s).map_err(std::io::Error::other)){Ok(v)=>v,Err(e)=>{eprintln!("input error: {e}");return ExitCode::FAILURE;}}}else{GenesisIntent::aster()};
 match genesis(intent){Ok(r)=>{match serde_json::to_string_pretty(&r){Ok(s)=>println!("{s}"),Err(e)=>{eprintln!("output error: {e}");return ExitCode::FAILURE;}}ExitCode::SUCCESS},
 Err(e)=>{eprintln!("genesis failed: {e}");ExitCode::FAILURE}}
}
