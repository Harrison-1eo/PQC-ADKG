use std::env;
use adkg::run;

pub fn main() {
    let args: Vec<String> = env::args().collect();
    
    let (n, f) = get_args(args);
    println!("n: {}, f: {}", n, f);
    
    // 运行协议，参数 `n` 为参与方总数量，`f` 为恶意参与方数量
    run(n, f);
}

fn get_args(args: Vec<String>) -> (usize, usize) {
    let mut n = 0;
    let mut f = 0;
    for i in 0..args.len() {
        if args[i] == "-n" {
            n = args[i+1].parse::<usize>().unwrap();
        }
        if args[i] == "-f" {
            f = args[i+1].parse::<usize>().unwrap();
        }
    }

    if 3*f+1 > n {
        panic!("must have 3f+1 <= n");
    }

    if n < 4 {
        panic!("must have n >= 4");
    }

    (n, f)
}



#[cfg(test)]
mod tests {
    use adkg::run;

    #[test]
    fn t() {
        for _ in 0..3{
            let n = 35;
            let f = (n-1)/3;
            println!("n: {}, f: {}", n, f);
            run(n, f);
        }
    }
}