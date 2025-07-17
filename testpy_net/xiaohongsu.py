import json
from re import compile
from lxml.etree import HTML
from yaml import safe_load

import requests


class XiaoHSDownload:
    LINK = compile(r"https?://www\.xiaohongshu\.com/explore/\S+")
    SHARE = compile(r"https?://www\.xiaohongshu\.com/discovery/item/\S+")
    SHORT = compile(r"https?://xhslink\.com/[^\s\"<>\\^`{|}，。；！？、【】《》]+")
    ID = compile(r"(?:explore|item)/(\S+)?\?")
    headers = {
        "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36"
    }

    def __init__(self):
        pass

    def parsec(self, share_url):
        self.get_video_url(share_url)

    def get_video_url(self, share_url):
        res = requests.get(share_url, self.headers)
        html = HTML(res.text)
        scripts = html.xpath("//script/text()")
        ss = ""
        for script in scripts:
            ss = str(script)
            if ss.startswith("window.__INITIAL_STATE__"):
                ss = ss.lstrip("window.__INITIAL_STATE__=")
                break

        if not script:
            return ""

        print("----------------------")

        data = self.deep_get(safe_load(ss))
        print(f"data: {data}")

        title = data["title"]

        is_vedio = data["type"] == "video"
        video_url = ""
        if is_vedio:
            video_url = data["video"]["consumer"]["originVideoKey"]

        video_url = f"https://sns-video-bd.xhscdn.com/{video_url}"

        print(f"title: {title}, is_vedio: {is_vedio}, url: {video_url}")

        filename = f"./video/{title}.mp4"

        video_content = requests.get(url=video_url, headers=self.headers).content

        with open(filename, "wb") as f:
            f.write(video_content)

    def deep_get(self, data):
        if not data:
            return ""
        try:
            keys = ["note", "noteDetailMap", "[-1]", "note"]
            for key in keys:
                if key.startswith("[") and key.endswith("]"):
                    data = self.sefe_get(data, int(key[1:-1]))
                else:
                    data = data[key]
            return data
        except:
            return ""

    def sefe_get(self, data, index):
        if isinstance(data, dict):
            return list(data.values())[index]
        elif isinstance(data, list | tuple | set):
            return data[index]
        raise TypeError


if __name__ == "__main__":
    url = "https://www.xiaohongshu.com/explore/6860fc4600000000120326b3?xsec_token=AB9QhIhpC8HhEOMKRMQafg9yOkQWjZ-E6a_T8OxQzvfz4=&xsec_source=pc_feed"
    XiaoHSDownload().parsec(url)
