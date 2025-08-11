use std::collections::HashMap;
use std::{env, fs};
use std::io::{Result, Write};

#[derive(Copy, Clone, Debug, PartialEq)]
enum CommandType {
    Arithmetic,
    Branching,
    Function,
    Push,
    Pop,
}


#[derive(Debug, PartialEq)]
struct LineParsing {
    ctype: CommandType,
    arg1: String, // This is either the arithmetic type, or the location
    loc: Option<String>,
    arg2: Option<i32>, // This will be the target of offsets or the num
    line_num: i32,
    file_name: String,
}

impl LineParsing {
    fn new(line: Vec<&str>, line_num: i32, file_name: String) -> Self {
        let mapping = HashMap::from([
            ("push", CommandType::Push),
            ("pop",  CommandType::Pop),
            ("add",  CommandType::Arithmetic),
            ("sub",  CommandType::Arithmetic),
            ("eq",   CommandType::Arithmetic),
            ("lt",   CommandType::Arithmetic),
            ("gt",   CommandType::Arithmetic),
            ("neg",  CommandType::Arithmetic),
            ("and",  CommandType::Arithmetic),
            ("or",   CommandType::Arithmetic),
            ("not",  CommandType::Arithmetic),
            ("label", CommandType::Branching),
            ("if-goto", CommandType::Branching),
            ("goto", CommandType::Branching),
            ("function", CommandType::Function),
            ("call", CommandType::Function),
            ("return", CommandType::Function),
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
            ("constant", "CONST"),
            ("temp", "TEMP"),
            ("pointer", "PTR"),
        ]);
        let ctype = mapping[line[0].trim()];
        let mut arg1 = line[0].to_string(); // Initalize to the line to ensure we never have a failure
        let mut arg2: Option<i32> = None;
        let mut loc: Option<String> = None;
        if  ctype == CommandType::Push || ctype == CommandType::Pop{
            loc = Some(location[line[1]].to_string()); // If len greater than 2 ctype will capture the first part of the argument, this will capture the label etc.
            arg2 = Some(line[2].parse().expect("Unable to parse num"));
        }
        else if ctype == CommandType::Branching {
            arg1 = line[0].to_string();
            loc = Some(line[1].to_string());
        }
        else if ctype == CommandType::Function {
            arg1 = line[1].to_string();
            arg2 = Some(line[2].parse()
                        .expect("Unable to parse num"));
        }
        else if line.len() >= 3 {
            arg2 = Some(line[2].parse()
                        .expect("Unable to parse num"));
        }
        LineParsing{ ctype, arg1, loc, arg2, line_num, file_name }
    }

    fn parse(&self) -> String {
        match self.ctype {
            CommandType::Pop => self.pop(),
            CommandType::Push => self.push(),
            CommandType::Arithmetic => self.arithmitic(),
            CommandType::Branching => self.branching(),
            CommandType::Function => self.function_parse(),
        }
    }

    fn branching(&self) -> String {
        match self.arg1.as_str() {
            "label" => self.label(),
            "goto" => self.goto(),
            "if-goto" => self.if_goto(),
            _ => panic!("Unable to parse item due to error in branching command: {0}", self.arg1),
        }
    }

    fn function_parse(&self) -> String {
        match self.arg1.as_str() {
            "function" => self.init_function().to_string(),
            "call" => "".to_string(),
            "return" => "".to_string(),
            _ => panic!("Unable to parse function call")
        }
    }

    fn init_function(&self) -> String {
        format!("({0}_return_address)
@({0}_return_address)
D=A
@SP
A=M
M=D //Jump to address saved
@SP
M=M+1
@LCL
D=M
@SP
A=M
M=D
@SP
M=M+1

@ARG
D=M
@SP
A=M
M=D
@SP
M=M+1

@THIS
D=M
@SP
A=M
M=D
@SP
M=M+1

@THAT
D=M
@SP
A=M
M=D
@SP
M=M+1

@5
D=A
@SP
D=M-D
@{2:?}
D=D-A
@ARG
M=D

@SP
D=M
@LCL
M=D

@{1}
0;JMP", self.line_num, self.arg1, self.arg2)
    }

    fn label(&self) -> String{
        format!("({0})", self.loc.clone().unwrap())
    }

    fn goto(&self) -> String{
        format!("@{0}
0;JMP", self.loc.clone().unwrap())
    }

    fn if_goto(&self) -> String{
        format!("@SP
AM=M-1
D=M
@{0}
D;JNE", self.loc.clone().unwrap())
    }

    fn arithmitic(&self) -> String {
        match self.arg1.as_str() {
            "add" => self.add(),
            "sub" => self.sub(),
            "eq"  => self.eq(),
            "lt"  => self.lt(),
            "gt"  => self.gt(),
            "neg" => self.neg(),
            "and" => self.and(),
            "or"  => self.or(),
            "not" => self.not(),
            _     => "".to_string(),
        }
    }
    // Tested
    fn add(&self) -> String {
        "@SP
AM=M-1
D=M
A=A-1
M=D+M".to_string()
    }

    fn sub(&self) -> String {
        "@SP
AM=M-1
D=M
A=A-1
M=M-D".to_string()
    }

    fn eq(&self) -> String {
        format!("@SP
AM=M-1
D=M
A=A-1
D=M-D
@true{0}
D;JEQ
@SP
A=M-1
M=0
@end{0}
D;JMP
(true{0})
@SP
A=M-1
M=-1
(end{0})", self.line_num)
    }

    fn lt(&self) -> String {
        format!("@SP
AM=M-1
D=M
A=A-1
D=M-D
@true{0}
D;JLT
@SP
A=M-1
M=0
@end{0}
D;JMP
(true{0})
@SP
A=M-1
M=-1
(end{0})", self.line_num)
    }

    fn gt(&self) -> String {
format!("@SP
AM=M-1
D=M
A=A-1
D=M-D
@true{0}
D;JGT
@SP
A=M-1
M=0
@end{0}
D;JMP
(true{0})
@SP
A=M-1
M=-1
(end{0})", self.line_num)
    }

    fn neg(&self) -> String {
"@SP
A=M-1
M=-M".to_string()
    }

    fn and(&self) -> String {
        "@SP
AM=M-1
D=M
A=A-1
M=D&M".to_string()
    }

    fn or(&self) -> String {
        "@SP
AM=M-1
D=M
A=A-1
M=D|M".to_string()
    }

    fn not(&self) -> String {
        "@SP
A=M-1
M=!M".to_string()
    }

    fn push(&self) -> String {
        if self.loc == None {
            panic!("Unable to parse due to lack of location")
        }
        match &self.loc.as_ref().unwrap()[..] {
            "CONST" => self.push_constant(),
            "LCL" | "ARG" | "THIS" | "THAT" => self.push_offset(),
            "TEMP" => self.push_temp(),
            "PTR" => self.push_pointer(),
            "STATIC" => self.push_static(),
            _ => panic!("Called a push command that is not implemented"),
        }
    }

    fn pop(&self) -> String {
        if self.loc == None {
            panic!("Unable to parse due to lack of location")
        }
        match &self.loc.as_ref().unwrap()[..] {
            "CONST" => panic!("Can't pop static items"),
            "LCL" | "ARG" | "THIS" | "THAT" => self.pop_offset(),
            "TEMP" => self.pop_temp(),
            "PTR" => self.pop_pointer(),
            "STATIC" => self.pop_static(),
            _ => panic!("Called a pop command that is not implemented"),
        }
    }

    fn push_constant(&self) -> String {
       format!("@{0}
D=A
@SP
A=M
M=D
@SP
M=M+1", self.arg2.unwrap())
    }

    fn push_static(&self) -> String {
        format!("@{0}.{1}
D=M
@SP
A=M
M=D
@SP
M=M+1", self.file_name, self.arg2.unwrap())
    }

    fn push_pointer(&self) -> String {
       let segment = match self.arg2.unwrap() {
           0 => "THIS",
           1 => "THAT",
           _ => panic!("Invalid pointer")
       };
        format!("@{0}
D=M
@SP
A=M
M=D
@SP
M=M+1", segment)
    }

    fn pop_static(&self) -> String {
        format!("@SP
AM=M-1
D=M
@{0}.{1}
M=D", self.file_name, self.arg2.unwrap())
    }

    fn pop_pointer(&self) -> String {
        let segment = match self.arg2.unwrap() {
            0 => "THIS",
            1 => "THAT",
            _ => panic!("Invalid pointer")
        };
        format!("@SP
AM=M-1
D=M
@{segment}
M=D")
    }

    fn pop_offset(&self) -> String {
        format!("@{0}
D=A
@{1}
D=D+M
@R13
M=D
@SP
AM=M-1
D=M
@R13
A=M
M=D", self.arg2.unwrap(), self.loc.clone().unwrap())
    }

    fn pop_temp(&self) -> String {
        format!("@SP
AM=M-1
D=M
@R{0}
M=D", self.arg2.unwrap() + 5)
    }

    fn push_offset(&self) -> String {
        format!("@{0}
D=A
@{1}
A=M
A=D+A
D=M
@SP
A=M
M=D
@SP
M=M+1", self.arg2.unwrap(), self.loc.clone().unwrap())
    }

    fn push_temp(&self) -> String {
        format!("@R{0}
D=M
@SP
A=M
M=D
@SP
M=M+1", self.arg2.unwrap() + 5)
    }


}

fn split_line(line: &str) -> Vec<&str> {
    line.trim()
        .split(" ")
        .collect()
}

pub fn open_line_breaks(filename: &str) -> String {
    fs::read_to_string(filename)
        .expect("Should have been able to read the file")

}

pub fn write_file (filename: &str, contents: &str) -> Result<()> {
    let mut file = fs::File::create(filename)?;
    let _ = file.write_all(contents.as_bytes());
    Ok(())
}

pub fn clean_whitespace(line: &str) -> Option<&str> {
    // We can split the line at // and select the first section to remove comments
    let vals = line.split("//").collect::<Vec<&str>>();
    let tmp = vals[0].trim();
    if !tmp.is_empty() {
        return Some(tmp)
    }
    None
}

fn isolate_filename(filepath: &str) -> String {
    let mut array = filepath.split("/").peekable();
    let mut filepath = Vec::new();
    while array.peek() != None {
        let tmp = array.next().unwrap();
        if array.peek() == None {
            let mut filename = tmp.splitn(2, ".");
            filepath.push(format!("{}", filename.next().unwrap()));
        }
    }
    let res = filepath.join("/");
    return res.to_string();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut file = "resources/Test.vm";
    if args.len() > 1 {
        file = &args[1];
    }
    let contents = open_line_breaks(file);
    let filename = isolate_filename(&file);
    let lines = contents.split("\n");
    let mut res = Vec::new();
    let mut line_num = 0;
    for line in lines {
        let tmp = clean_whitespace(line);
        println!("{:?}", tmp);
        if tmp.is_some() {
            line_num += 1;
            res.push(LineParsing::new(split_line(line), line_num, filename.clone())
                     .parse());
        }
    }
    let output = res.join("\n");
    println!("{:?}", output);
    let _ = write_file(&format!("./resources/{filename}.asm"), &output);
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

    #[test]
    fn integration_tesst0() {
        let line = "pop local 1";
        let c = LineParsing::new(split_line(line)).parse();
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
