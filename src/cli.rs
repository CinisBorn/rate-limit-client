use std::{env::args, process};

#[derive(Debug)]
pub struct CliArgs {
    pub endpoint: String, 
    pub req_amount: i32, // amount request by some time unit measurement
}

#[derive(Debug, PartialEq)]
enum CliErrors {
    InvalidArgumentFormat,
    MissingArguments,
    EmptyArguments, 
}

impl CliArgs {
    pub fn get_args() -> Self {
        let mut input: Vec<String> = args().collect();
        let usr_args = input.drain(1..).collect::<Vec<_>>();
        
        match CliArgs::format_usr_args(usr_args) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("ERROR[{:?}]", e);
                process::exit(1);
            }
        }
    }
    
    fn format_usr_args(args: Vec<String>) -> Result<Self, CliErrors> {
        if args.len() < 2 {
            return Err(CliErrors::MissingArguments);
        }
        
        if args.iter().any(|a| a.trim() == ""){
            return Err(CliErrors::EmptyArguments);
        }
        
        let freq = args[1].parse::<i32>().map_err(|_| {
            CliErrors::InvalidArgumentFormat
        })?;
        
        Ok(Self {
            endpoint: args[0].clone(),
            req_amount: freq,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn cli_get_args() {
        let freq = 10.to_string();
        let endpoint = "http://localhost".to_string();
        let args = CliArgs::format_usr_args(vec![endpoint.clone(), freq.clone()]).unwrap();
        
        assert_eq!(args.endpoint, endpoint);
        assert_eq!(args.req_amount.to_string(), freq);
    }
    
    #[test] 
    fn reject_missing_args() {
        let endpoint = "http://localhost".to_string();
        let args = CliArgs::format_usr_args(vec![endpoint.clone()]);
        
        assert_eq!(args.unwrap_err(), CliErrors::MissingArguments);
    }
    
    #[test]
    fn reject_invalid_args() {
        let freq = "abc".to_string();
        let endpoint = "http://localhost".to_string();
        let args = CliArgs::format_usr_args(vec![endpoint, freq]);
        
        assert_eq!(args.unwrap_err(), CliErrors::InvalidArgumentFormat);
    }
    
    #[test]
    fn reject_empty_args() {
        let freq = "".to_string();
        let endpoint = "http://localhost".to_string();
        let args = CliArgs::format_usr_args(vec![endpoint, freq]);
        
        assert_eq!(args.unwrap_err(), CliErrors::EmptyArguments);
    }
}