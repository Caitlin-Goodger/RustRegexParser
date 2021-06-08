use std::env;
use std::fs;
use Regex::*;

/**
 * Type of Symbols possibly in the Expression
 */
#[derive(Eq, PartialEq, Debug)]
pub enum Regex {
    Or(Box<Regex>, Box<Regex>),
    Pattern(Vec<Box<Regex>>),
    Star(Box<Regex>),
    Character(char),
    Syn(char),
}


/**
 * Struct of the Parser
 * It has a string that it will parse and a position along the string that it is currently at
 */
pub struct Parser {
    expression: String,
    pos: usize
}

/**
 * Implementation of the Parser that will parse the string provided
 */
impl Parser {
    pub fn new(ex: String) -> Parser {
        Parser { expression: ex, pos: 0 }
    }

    /**
     * Parse method. This is where the parsing starts from
     * Since this is a recusive decent parser top symbol is the |
     */
    fn parse(&mut self) -> Result<Regex, &'static str> {
        let t1 = self.parse_pattern()?;
        if self.expression.len() > self.pos && self.get_next() == '|' {
            self.pos = self.pos + 1;
            let t2 = self.parse_pattern()?;
            return Ok(Or(Box::new(t1), Box::new(t2)))
        } else {
            return Ok(t1)
        }
    }

    /**
     * Next Parsing method down
     * Parses is there is not operator between characters
     * E.g. between a and b in ab
     */
    fn parse_pattern(&mut self) -> Result<Regex, &'static str> {
        let mut v = Vec::new();
        while self.expression.len() > self.pos && self.get_next() != ')' && self.get_next() != '|' {
            let f = self.parse_star()?;
            v.push(Box::new(f));
        }
        if self.expression.len() > self.pos && self.get_next() == '|' {
            self.pos = self.pos + 1;
            let t2 = self.parse_pattern()?;
            return Ok(Or(Box::new(Pattern(v)), Box::new(t2)))
        }
        return Ok(Pattern(v))
    }

    /**
     * Next Parsing method down
     * Parses the * character
     */
    fn parse_star(&mut self) -> Result<Regex, &'static str> {
        let b = self.parse_single()?;
        if self.expression.len() > self.pos && self.get_next() == '*' {
            self.pos = self.pos + 1;
            return Ok(Star(Box::new(b)))
        } else {
            return Ok(b)
        }
    }

    /**
     * Next Parsing method down
     * Parses the character and dot symbols
     */
    fn parse_single(&mut self) -> Result<Regex, &'static str> {
        if self.get_next() == '(' {
            self.pos = self.pos + 1;
            let r = (self.parse())?;
            self.pos = self.pos + 1;
            return Ok(r)
        } else {
            let c = self.get_next();
            self.pos = self.pos + 1;
            return Ok(Character(c))
        }
    }
    
    /**
     * Get the next item from the expression string
     */
    fn get_next(&self) -> char {
        let char_vec: Vec<char> = self.expression.chars().collect();
        return char_vec[self.pos];
    }
}

/**
 * Check that there is an even number of opening and closing brackets
 */
fn brackets(b: String) -> bool {
    let char_vec: Vec<char> = b.chars().collect();
    let mut count = 0;
    for c in char_vec {
        if c == '(' {
            count = count + 1;
        }
        if c == ')' {
            count = count - 1;
        }
    }
    return count == 0
}

/**
 * Check that expression is in the right format
 */
fn right_format(b: String) -> bool {
    let char_vec: Vec<char> = b.chars().collect();
    let si = b.len();
    let mut las = char_vec[0];
    if las == '*' {
        return false;
    }
    for x in 1..si {
        let v = char_vec[x];
        if v == '*' {
            if las == '|' || las == '(' {
                return false
            }
        }
        las = v;
    }
    return true
}

/**
 * Match the expression to the target string to see if it matches
 */
fn match_target(mut target: String, expression: &Regex, ex: String, is_st: bool, character: char) -> (String, String) {
    match expression {
        Character(c) => {
            let char_vec: Vec<char> = target.chars().collect();
            if target.len() == 0 {
                return ("NO".to_string(), target)
            }
            if char_vec[0] == *c {
                let _r = target.remove(0);
                ("YES".to_string(), target)
            } else if *c == '.' {
                let _r = target.remove(0);
                ("YES".to_string(), target)
            }else {
                ("NO".to_string(), target)
            }
        }
        Pattern(v) => {
            if v.is_empty() {
                if target.len() == 0 {
                    return ("YES".to_string(),target)
                } else {
                    return ("NO".to_string(),target)
                }
            }
            let mut r = "YES".to_string();
            let mut t_target = target.clone();
            let mut counter = 0;
            let mut is_s;
            let mut ch = 'b';
            //Explore all the Elements in the List
            for b in v {
                let m = b.clone();
                let s = is_star(m);
                is_s = s.0;
                //Find out what the next character is
                if counter < v.len() && is_s{
                    let n = v.get(counter + 1);
                    match n {
                        Some(s) => {
                            let c = get_next_char(s);
                            if c.0 {
                                ch = c.1
                            }
                            
                        }
                        None => ch = character
                    }
                }
                let result = match_target(t_target, b,ex.clone(),is_s, ch);
                if result.0 == "NO".to_string() {
                    r = result.0;
                    return (r, target)
                }
                t_target = result.1;
                counter = counter + 1;
            }
            return (r, t_target)
        }
        Or(l,r) => {
            let l_result = match_target(target.clone(), l, ex.clone(), false, 'n');
            let r_result = match_target(target.clone(), r, ex.clone(), false, 'n');
            //If both the left and right return yes, return the one that is closer to matching the entire target
            if l_result.0 == r_result.0 && l_result.0 == "YES".to_string() {
                let l_tar = l_result.1;
                let r_tar = r_result.1;
                if r_tar.len() < l_tar.len() {
                    return (r_result.0, r_tar)
                } else {
                    return (l_result.0, l_tar)
                }
            }
            if l_result.0 == "YES".to_string() {
                return (l_result.0, l_result.1)
            } else if r_result.0 == "YES".to_string() {
                return (r_result.0, r_result.1)
            }

            return("NO".to_string(), target)
        }
        Star(r) => {
            if target.len() == 0 {
                return ("YES".to_string(), target)
            }
            let mut ret = "YES".to_string();
            let mut t_target = target.clone();
            let star_p = star_dot(target, r, ex.clone(), is_st, character);
            if star_p.2 {
                return ("YES".to_string(), star_p.1)
            }
            //Keep matching until reaching the end of the target or it doesn't match anymore
            while ret == "YES".to_string() && t_target.len() > 0 {
                let t = match_target(t_target.clone(), r, ex.clone(), false, 'n');
                let t_r = t.0;
                let t_t = t.1;
                t_target = t_t;
                ret = t_r;
            }
            
            return ("YES".to_string(), t_target)
        }
        //Type for a SYNTAX ERROR
        Syn(_e) => return ("SYNTAX ERROR".to_string(), "".to_string()),
    }
}

/** 
 * Check if the current element in the expression is a Star 
 * Or if it is a Pattern, check is the last element is a Star
 */
fn is_star(expression: &Regex) -> (bool, String) {
    match expression {
        Star(_s) => (true, "Star".to_string()),
        Pattern(v) => {
            let m = v.get(v.len()-1);
            match m {
                Some(b) => {
                    let r = is_star(b);
                    if r.0 {
                        return (true, "Pattern".to_string())
                    }
                    return (false, "Pattern".to_string())
                }
                None => return (false, "Pattern".to_string())
            }
            
        }
        _ => (false, "Other".to_string())
    }
}

/**
 * Get the next character is the element is a Character
 */
fn get_next_char(expression: &Regex) -> (bool,char) {
    match expression {
        Character(c) => return (true,*c),
        _ => return (false,'e')
    }
}

/**
 * If it is a .*, that can be repeated as many times as it wants
 * Keep reading the target until either, reaching the end of the expression 
 * or find the character that is the next character in the expression
 */
fn star_dot(target: String, expression: &Regex, ex: String, is_star: bool, ch : char) -> (String, String, bool) {
    match expression {
        Character(c) => {
            let ret = "YES".to_string();
            let mut t_target = target.clone();
            let mut char_vec: Vec<char> = t_target.chars().collect();
            let mut return_bool = false;
            if *c == '.' {
                let fir = char_vec[0];
                let mut curr = fir.clone();
                return_bool = true;
                let mut count_c = count_ch(target, ch);
                let c_ex = count_ch(ex, ch);
                while ret == "YES".to_string() && t_target.len() > 0 && (curr != ch || count_c > c_ex) && is_star {
                    let r = t_target.remove(0);
                    if r == ch {
                        count_c = count_c -1;
                    }
                    let _c = char_vec.remove(0);
                    if t_target.len() > 0 {
                        curr = char_vec[0];
                    }
                }   
            }
            return ("YES".to_string(), t_target, return_bool);
        }
        _ => {
            return ("YES".to_string(), target, false)
        }
    }
}


/**
 * Count how many of a specified character is in the expression
 */
fn count_ch (target: String, ch: char) -> i32 {
    let char_vec: Vec<char> = target.chars().collect();
    let mut count = 0;
    for c in char_vec {
        if c == ch {
            count = count + 1;
        }
    }
    return count
}


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        panic!("Not the right number of arguments");
    }

    let ex_file_name = &args[1];
    let tar_file_name = &args[2];

    let expressions = fs::read_to_string(ex_file_name).expect("Something went wrong reading the file");

    let targets = fs::read_to_string(tar_file_name).expect("Something went wrong reading the file");
    let mut counter = 0;
    let ex_lines = expressions.lines();
    let mut parsed_ex: Vec<Regex> = Vec::new();
    let mut ex: Vec<String> = Vec::new();
    //Reach and Parse the Expressions
    for line in ex_lines { 
        let count = brackets(line.to_string());
        let form = right_format(line.to_string());
        ex.push(line.to_string());
        if !count || !form {
            let temp = Syn('e');
            parsed_ex.push(temp);
        } else {
            let mut p = Parser::new(line.to_string());
            let q = p.parse();
            match q {
                Ok(f) => parsed_ex.push(f),
                _ => println!("SYNTAX ERROR"),

            }
        }
        
        counter = counter +1;
    }
    
    //Read then Match the targets
    for line in targets.lines() {
        let target = line.to_string();
        let expression = parsed_ex.remove(0);
        let ex = ex.remove(0);
        let result = match_target(target,&expression, ex,false, 'n');
        let tar = result.1;
        if tar.len() > 0 {
            println!("{}", "NO".to_string()); 
        } else {
            println!("{}", result.0); 
        }
    }
}
