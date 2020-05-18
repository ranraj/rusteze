use std::process::Command;

pub fn connect_terminal(user_name:String,host:String) -> std::io::Result<()>{  
    Command::new("osascript")            
            .arg("-e")
            .arg(format!("tell application \"Terminal\" to do script \"exec ssh {}@{}\"",user_name,host))
            .output()?;
    Ok(())        
}