import os
import requests

import re

import json


def parsec(url):
    cookie = ""
    headers = {
        "Referer": url,
        "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
        "Cookie": cookie,
    }

    response = requests.get(url=url, headers=headers)
    print(response.url)
    # print(html)
    url = response.url.replace("/?", "?") # 适配新情况，不知道是否是临时问题
    print(url)
    headers = {
        "Referer": url,
        "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
        "Cookie": cookie,
    }
    response = requests.get(url=url, headers=headers)
    html = response.text
    print(response.url)

    title = re.findall('title="(.*?)"', html)[0]
    print(title)

    info = re.findall("window.__playinfo__=(.*?)</script>", html)[0]

    print(info)

    json_data = json.loads(info)

    video_url = json_data["data"]["dash"]["video"][-1]["baseUrl"]
    print(video_url)

    audio_url = json_data["data"]["dash"]["audio"][-1]["baseUrl"]
    print(audio_url)
    video_content = requests.get(url=video_url, headers=headers).content

    audio_content = requests.get(url=audio_url, headers=headers).content

    file = "video"
    if not os.path.exists(file):
        os.mkdir(file)

    with open("video\\" + title + ".mp4", mode="wb") as v:
        v.write(video_content)
    with open("video\\" + title + ".mp3", mode="wb") as a:
        a.write(audio_content)


if __name__ == "__main__":
    # url = "https://b23.tv/wqb0FAy"
    # url = "https://www.bilibili.com/video/BV1vd3sz3ERN/?share_source=copy_web"
    url = "https://b23.tv/kuWFjoV"
    # url = "https://b23.tv/LVLGJRW"
    parsec(url)
