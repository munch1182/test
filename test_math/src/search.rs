use std::iter;

use pinyin::{Pinyin, to_pinyin_vec};

fn _conver_to_pinyin<S: ToString>(data: &Vec<S>) -> Vec<String> {
    return data
        .iter()
        .map(|s| {
            let str = to_pinyin_vec(&s.to_string(), Pinyin::plain);
            if str.len() > 0 {
                str.concat()
            } else {
                s.to_string()
            }
        })
        .collect::<Vec<_>>();
}

pub fn score_query(query: &str, target: &Vec<&str>) -> Vec<f32> {
    return target.iter().map(|s| n_gram(query, s)).collect::<Vec<_>>();
}

pub fn sort_query<'a>(query: &str, target: &Vec<&'a str>) -> Vec<&'a str> {
    let mut res = target.to_vec();
    res.sort_by(|a, b| {
        n_gram(query, b)
            .partial_cmp(&n_gram(query, a))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    res
}

#[allow(dead_code)]
fn n_gram(a: &str, b: &str) -> f32 {
    let str_len = a.chars().count() + 1;

    let trigrams_a = _trigrams_impl(a);
    let trigrams_b = _trigrams_impl(b);

    let mut score = 0f32;

    for t_a in &trigrams_a {
        for t_b in &trigrams_b {
            if t_a == t_b {
                score += 1f32;
                break;
            }
        }
    }
    let res = score / str_len as f32;
    if 0f32 <= res && res <= 1f32 {
        res
    } else {
        0f32
    }
}

fn _trigrams_impl(s: &str) -> Vec<(char, char, char)> {
    let it_1 = iter::once(' ').chain(iter::once(' ')).chain(s.chars());
    let it_2 = iter::once(' ').chain(s.chars());
    let it_3 = s.chars().chain(iter::once(' '));

    it_1.zip(it_2)
        .zip(it_3)
        .map(|((c1, c2), c3)| (c1, c2, c3))
        .collect()
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn it_works() {
        let list = _test();
        let query = "exc";
        let start = std::time::Instant::now();
        let res = sort_query(query, &list);
        println!("find {query}, res: {:?}, cost: {:?}", res, start.elapsed());
        assert!(res[1] == "exx" && res[0] == "excaaxa");
    }

    fn _test() -> Vec<&'static str> {
        vec![
            "exx",
            "aaa",
            "maybe",
            "e1x2",
            "window progress",
            "rust progress",
            "explore progress",
            "example.exe",
            "excaaxa",
        ]
    }

    #[test]
    fn test_pinyin() {
        let data = vec!["星期一", "行不行", "新加坡", "心情好", "nihaoma", "?78"];
        let new_data = _conver_to_pinyin(&data);
        let result = data
            .iter()
            .zip(new_data.iter())
            .map(|(x, y)| format!("{x}: {y}"))
            .collect::<Vec<String>>();
        println!("result: {:?}", result);
        assert!(new_data.iter().all(|x| x.len() > 0));
    }
}
