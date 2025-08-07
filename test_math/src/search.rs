use std::cmp::max;

use pinyin::ToPinyin;

/**
 * 返回模糊搜索得分
 *
 * 分数只在一次比较中有效，不能跨不同的查询参数来比较分数
 */
pub fn fuzzy_score<S: ToString>(query: &S, data: &S) -> u32 {
    let query = query.to_string().to_lowercase();
    // 处理空查询
    if query.is_empty() {
        return 0;
    }
    let data = data.to_string().to_lowercase();

    // 转换为字符向量以便索引访问
    let q_chars: Vec<char> = query.chars().collect();
    let d_chars: Vec<char> = data.chars().collect();

    // 存储第一个查询字符在目标中的所有位置
    let mut start_indices = Vec::new();
    for (i, &c) in d_chars.iter().enumerate() {
        if c == q_chars[0] {
            start_indices.push(i);
        }
    }

    let mut best_score = 0;

    // 遍历所有可能的起始位置
    for &start in &start_indices {
        let mut score = 0;
        // 第一个字符的得分
        score += 1; // 基础分
        if start == 0 {
            score += 5; // 开头位置奖励
        }

        let mut last_match = start; // 上一个匹配位置
        let mut q_index = 1; // 下一个要匹配的查询字符索引

        // 遍历目标字符串剩余部分
        for i in (start + 1)..d_chars.len() {
            if q_index >= q_chars.len() {
                break; // 已匹配所有查询字符
            }

            if d_chars[i] == q_chars[q_index] {
                // 基础分
                score += 1;

                // 连续性奖励（当前字符紧跟前一个匹配字符）
                if last_match == i - 1 {
                    score += 3;
                }

                last_match = i;
                q_index += 1;
            }
        }

        best_score = max(best_score, score);
    }

    best_score
}

/**
 * 返回数据列表中的对应得分
 */
pub fn fuzzy_search_score<'a, S: ToString>(query: &S, data: &Vec<S>) -> Vec<u32> {
    let query = query.to_py();
    data.iter()
        .map(|x| fuzzy_score(&query, &x.to_py()))
        .collect::<Vec<u32>>()
}

/**
 * 模糊搜索并返回排序后的字符
 */
pub fn fuzzy_search<'a, S: ToString>(query: &S, data: &Vec<S>) -> Vec<String> {
    let mut new_map = fuzzy_search_score(query, data)
        .iter()
        .zip(data.iter())
        .map(|(score, str)| (str, *score))
        .collect::<Vec<(&S, u32)>>();
    new_map.sort_by(|(_, a_score), (_, b_score)| b_score.cmp(&a_score));
    new_map
        .iter()
        .map(|(str, _)| str.to_string())
        .collect::<Vec<String>>()
}

// todo 用于实现类型比较
pub trait FuzzyScore {
    fn caluate(&self, query: &impl FuzzyScore) -> u32;
}

trait ToPinyinExt {
    /**
     * 转成拼音
     */
    fn to_py(&self) -> String;
}

impl<S: ToString> ToPinyinExt for S {
    fn to_py(&self) -> String {
        let str = self.to_string();
        str.chars()
            .map(|x| match x.to_pinyin() {
                Some(p) => p.plain().to_string(),
                None => x.to_string(),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_matching() {
        let query = "yz";
        let data = vec![
            "就依他说的这么办吧",
            "啊这",
            "ez",
            "医者1仁心",
            "伊泽瑞尔",
            "zyyz",
        ];
        let result = fuzzy_search_score(&query, &data);
        println!("search: {query} => {:?}", result);
        let result = fuzzy_search(&query, &data);
        println!("search: {query} => {:?}", result);
        assert!(result[2] == "zyyz")
    }

    #[test]
    fn test_py() {
        let str = "医者1仁心 ".to_py();
        println!("{str}");
        assert_eq!(str, "yizhe1renxin ".to_string())
    }
}
