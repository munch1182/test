import json
import os
import re
from urllib.parse import parse_qs, urlparse
import requests
from re import compile


class KuaishouDownload:
    SHORT_URL = compile(
        r"(https?://\S*kuaishou\.(?:com|cn)/[^\s\"<>\\^`{|}，。；！？、【】《》]+)"
    )
    LIVE_URL = compile(r"https?://live\.kuaishou\.com/\S+/\S+/(\S+)")
    PC_COMPLETE_URL = compile(r"(https?://\S*kuaishou\.(?:com|cn)/short-video/\S+)")
    REDIRECT_URL = compile(r"(https?://\S*chenzhongtech\.(?:com|cn)/fw/photo/\S+)")
    HEADERS = {
        "Referer": "https://www.kuaishou.cn/new-reco",
        "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36",
    }
    SCRIPT = "//script/text()"
    WEB_KEYWORD = "window.__APOLLO_STATE__="
    APP_KEYWORD = "window.INIT_STATE = "
    PHOTO_REGEX = compile(r"\"photo\":(\{\".*\"}),\"serialInfo\"")

    def parsec(self, url):

        res = requests.get(url, headers=self.HEADERS, allow_redirects=True)

        url_redirect = res.url
        c = res.cookies

        cookie = c.items()
        cookie_str = "; ".join([f"{key}={value}" for key, value in cookie])

        print("url_redirect: ", url_redirect)
        # 如果返回的参数没有对应数值，则是cookie中的did无效，需要去注册该did
        print("cookie: ", cookie_str)

        web, detail_id = self.detail_id(url_redirect)
        if not detail_id:
            print("fail parsec")
            return

        print("detail_id: ", detail_id)

        title, url = self.parse_data(res.text, web, detail_id)

        if not title or not url:
            print("fail parse_data")
            return

        # print("---------------------")

        # headers = self.HEADERS.copy()
        # headers["Cookie"] = cookie_str
        # res2 = requests.get(url, headers=headers, allow_redirects=True)

        # self.parse_data(res2.text, web, detail_id)

        video_content = requests.get(url=url, headers=self.HEADERS).content

        file = "video"
        if not os.path.exists(file):
            os.mkdir(file)

        with open("video\\" + title + ".mp4", mode="wb") as v:
            v.write(video_content)

    def parse_data(self, html, web, detail_id):
        script = self.script(html, web)

        print(script)

        name = f"VisionVideoDetailPhoto:{detail_id}"

        if not script.__contains__(name):
            print("not find VisionVideoDetailPhoto")
            return ("", "")

        json_data = json.loads(script)

        print(json_data)

        detail = json_data["defaultClient"][name]

        url = detail["photoUrl"]
        title = detail["caption"]
        print(url)
        return (title, url)

    def script(self, html, web):
        find = ""
        if web:
            find1 = re.findall("window.__APOLLO_STATE__=(.*?)</script>", html)
            if find1:
                find = find1[0]
        else:
            find2 = re.findall("window.INIT_STATE = (.*?)</script>", html)
            if find2:
                find = find2[0]
        return str(
            find.replace(
                ";(function(){var s;(s=document.currentScript||document.scripts["
                "document.scripts.length-1]).parentNode.removeChild(s);}());",
                "",
            )
        )

    def detail_id(self, url):
        url = urlparse(url)
        params = parse_qs(url.query)
        if "chenzhongtech" in url.hostname:
            return (False, params.get("photoId", [""])[0])
        elif "short-video" in url.path:
            return (
                True,
                url.path.split("/")[-1],
            )
        else:
            return (False, "")


if __name__ == "__main__":
    url = "https://v.kuaishou.com/K03QpTh3"
    KuaishouDownload().parsec(url)
