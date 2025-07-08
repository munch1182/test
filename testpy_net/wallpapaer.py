import re
import requests
import os
# import random


def download(href):
    headers = {
        "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36 Edg/122.0.0.0"
    }
    file = "wallpaper"
    # 发送 GET 请求获取每个壁纸的详细页面内容，并将其存储在 resp 变量中
    resp = requests.get(href, headers=headers).text
    # 使用正则表达式从详细页面内容中提取图片的关键信息
    key = re.findall(r'<a href="/download/(.*?)"', resp)[0].split('/')[-1]
    # 从壁纸链接中提取图片的编号
    number = href.rsplit('/', 1)[-1]
    # 构建图片的文件名，包括编号和关键信息
    title = number + '_' + key + '.jpg'
    # 构建图片的完整链接
    # 定义图片的基础 URL
    t = "https://images.wallpaperscraft.ru/image/single/"
    href = t+title
    # 打印图片链接
    # print(href)
    # 发送 GET 请求获取图片内容，并将其存储在 resp 变量中
    resp = requests.get(href, headers=headers).content
    # 以二进制写入模式打开文件，并将图片内容写入文件
    with open(os.path.join(file, title), mode='wb') as f:
        f.write(resp)
        print("下载完成", title)


def first(urls):
    headers = {
        "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36 Edg/122.0.0.0"
    }
    for i in range(1, 21):
        url = "https://wallpaperscraft.ru/catalog/anime/page" + str(i)
        # 发送 GET 请求获取网页内容，并将其存储在 resp 变量中
        resp = requests.get(url, headers=headers).text
        # 使用正则表达式从网页内容中提取所有壁纸链接
        hrefs = re.findall(r'<a class="wallpapers__link" href="(.*?)"', resp)
        # 在每个壁纸链接前添加网站的基础 URL，以获取完整的链接
        hrefs = ["https://wallpaperscraft.ru" + i for i in hrefs]
        urls.extend(hrefs)


def main():
    # 定义存储壁纸的文件夹名称
    file = "wallpaper"
    # 如果文件夹不存在，则创建该文件夹
    if not os.path.exists(file):
        os.mkdir(file)
    urls = []
    first(urls)
    for href in urls:
        download(href)


if __name__ == '__main__':
    main()