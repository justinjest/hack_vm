use std::collections::HashMap;
use std::fs;
use std::io::{Result, Write};

#[derive(Copy, Clone, Debug, PartialEq)]
enum CommandType {
    Arithmetic,
    Push,
    Pop,
}


#[derive(Debug, PartialEq)]
struct LineParsing {
    ctype: CommandType,
    arg1: String, // This is either the arithmetic type, or the location
    arg2: Option<i32>, // This will be the target of offsets or the num
}

impl LineParsing {
    fn new(line: Vec<&str>) -> Self {
        let mapping = HashMap::from([
            ("push", CommandType::Push),
            ("pop", CommandType::Pop),
            ("add", CommandType::Arithmetic),
            ("sub", CommandType::Arithmetic),
        ]);

        let location = HashMap::from([
            ("local", "LCL"),
            ("argument", "ARG"),
            ("this", "THIS"),
            ("that", "THAT"),
            ("static", "STATIC"),
            ("R13", "R13"),
            ("R14", "R14"),
            ("R15", "R15"),
            ("constant", "CONSTANT"),
            ("temp", "TEMP"),
        ]);

        let ctype = mapping[line[0]];
        let mut arg1 = line[0].to_string();
        let mut arg2: Option<i32> = None;
        if line.len() >= 3 {
            print!("Original: {}\n", line[1]);
            println!("New: {}\n", location[line[1]]);
            arg1 = location[line[1]].to_string();
            arg2 = Some(line[2].parse()
                        .expect("Unable to parse num {line[2]}\n"));
        }
        LineParsing{ ctype: ctype, arg1: arg1, arg2: arg2 }
    }

    fn parse(&self) -> String {
        match self.ctype {
            CommandType::Pop => return self.pop(),
            CommandType::Push => return self.push(),
            CommandType::Arithmetic => return self.arithmitic(),
        }
    }

    fn arithmitic(&self) -> String {
        match self.arg1.as_str() {
            "add" => return self.add(),
            "sub" => return self.sub(),
            _ => return "".to_string(),
        }
    }

    fn push(&self) -> String {
        match &self.arg1[..] {
            "STATIC" => return self.push_constant(),
            "LCL" | "ARG" | "THIS" | "THAT" => return self.push_offset(),
            _ => return "".to_string(),
        }
    }

    fn pop(&self) -> String {
        match &self.arg1[..] {
            "STATIC" => panic!("Can't pop static items"),
            "LCL" | "ARG" | "THIS" | "THAT" => return self.pop_offset(),
            _ => return "".to_string(),
        }
    }

    fn add(&self) -> String {
        "@SP
M=M-1
A=M
D=M
A=A-1
M=D+M".to_string()
    }

    fn sub(&self) -> String {
        "@SP
M=M-1
A=M
D=M
A=A-1
M=M-D".to_string()
    }

    fn push_constant(&self) -> String {
        println!("{:?}", self);
       format!("@{:?}
D=A
@SP
A=M
M=D
@SP
M=M+1", self.arg2.unwrap()).to_string()
    }

    fn pop_offset(&self) -> String {
        format!("@{0}
D=A
@{1}
A=D+M
D=M
@SP
M=M-1
A=M
M=D", self.arg2.unwrap(), self.arg1).to_string()
    }

    fn push_offset(&self) -> String {
        format!("@{0}
D=A
@{1}
A=D+M
D=M
@SP
M=M+1
A=M
M=D", self.arg2.unwrap(), self.arg1).to_string()
    }

}

fn split_line(line: &str) -> Vec<&str> {
    line.split(" ")
        .collect()
}

pub fn open_line_breaks(filename: &str) -> String {
    let contents = fs::read_to_string(filename)
        .expect("Should have been able to read the file");
    return contents;
}

pub fn write_file (filename: &str, contents: &str) -> Result<()> {
    let mut file = fs::File::create(format!("{}", filename.to_string()))?;
    let _ = file.write_all(contents.as_bytes());
    Ok(())
}

pub fn clean_whitespace(line: &str) -> Option<&str> {
    // We can split the line at // and select the first section to remove comments
    let vals = line.split("//").collect::<Vec<&str>>();
    let tmp = vals[0].trim();
    if tmp != "" {
        return Some(tmp)
    }
    return None
}


fn main() {
    println!("Hello, world!");
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_line_push() {
        let c = split_line("push local 1");
        let e = ["push", "local", "1"];
        assert_eq!(c, e);
    }

    #[test]
    fn split_line_arithmetic() {
        let c = split_line("add");
        let e = ["add"];
        assert_eq!(c, e);
    }

    #[test]
    fn line_parsing_test0() {
        let tmp = Vec::from(["pop", "local", "1"]);
        let c = LineParsing::new(tmp);
        let e = LineParsing { ctype: CommandType::Pop,
        arg1: "LCL".to_string(),
        arg2: Some(1)};
        assert_eq!(c, e);
    }

    #[test]
    fn line_parsing_test1() {
        let tmp = Vec::from(["add"]);
        let c = LineParsing::new(tmp);
        let e = LineParsing {
            ctype: CommandType::Arithmetic,
            arg1: "add".to_string(),
            arg2: None,
        };
        assert_eq!(c, e);
    }

    #[test]
    fn line_test0() {
        let tmp = Vec::from(["push", "static", "1"]);
        let c = LineParsing::new(tmp).push();
        let e = "@1
D=A
@SP
A=M
M=D
@SP
M=M+1".to_string();
        assert_eq!(c, e);
    }

    #[test]
    fn line_test1() {
        let tmp = Vec::from(["pop", "local", "1"]);
        let c = LineParsing::new(tmp).pop();
        let e = "@1
D=A
@LCL
A=D+M
D=M
@SP
M=M-1
A=M
M=D".to_string();
        assert_eq!(c, e);
    }

}
