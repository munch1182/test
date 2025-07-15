import os
import urllib.parse
import requests
import re
import random
from tqdm import tqdm
import urllib
import execjs


class DouYinDownload:

    def __init__(self):
        self.headers = {
            "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3",
            "Referer": "https://www.douyin.com/",
        }

    def parsec(self, share_url):
        session = requests.session()
        req = session.head(share_url, headers=self.headers, allow_redirects=True)
        video_id = re.findall(r"/video/(\d+)", req.url)[0]

        print("video_id: ", video_id)
        params = {
            "aweme_id": video_id,
            "aid": 6383,
            "version_code": 190500,
            "version_name": "19.5.0",
            "device_platform": "android",
            "os_version": 6,
            "update_version_code": "1704000",
            "pc_client_type": 1,
        }

        url = "https://www.douyin.com/aweme/v1/web/aweme/detail/"

        msToken = self.msToken()
        params["msToken"] = msToken
        params["a_bogus"] = self.abogus(params)

        self.headers["Cookie"] = self.cookie(msToken)

        print("params: ", params)
        print("headers: ", self.headers)

        res = requests.get(url, params=params, headers=self.headers)

        print(res.status_code, res.url, len(res.json()) > 0)

        data = res.json()["aweme_detail"]

        title = data["desc"]
        url = data["video"]["play_addr"]["url_list"][0].replace("playwm", "play")
        print(f"{title}: {url}")
        self.download(title, url)
        return title, url

    def download(self, title, url):
        # 下载视频
        filename = f"./video/{title}.mp4"
        if not os.path.exists(filename):
            with open(filename, "wb") as f:
                f.write(requests.get(url, headers=self.headers).content)

    def abogus(self, params):
        with open("./abogus.js", encoding="utf-8") as f:
            js = execjs.compile(f.read())
            return js.call(
                "sign_datail",
                urllib.parse.urlencode(params),
                self.headers["User-Agent"],
            )

    def msToken(self, randomlength=120):
        random_str = ""
        base_str = "ABCDEFGHIGKLMNOPQRSTUVWXYZabcdefghigklmnopqrstuvwxyz0123456789="
        length = len(base_str) - 1
        for _ in range(randomlength):
            random_str += base_str[random.randint(0, length)]
        return random_str

    def cookie(self, msToken):
        ttwid = self.ttwid()
        return f"msToken={msToken}; ttwid={ttwid};"

    def ttwid(self):
        data = '{"region":"cn","aid":1768,"needFid":false,"service":"www.ixigua.com","migrate_info":{"ticket":"","source":"node"},"cbUrlProtocol":"https","union":true}'
        res = requests.request(
            "POST", "https://ttwid.bytedance.com/ttwid/union/register/", data=data
        )
        cookie = res.headers["Set-Cookie"]
        return re.findall("ttwid=(.*?);", cookie)[0]


if __name__ == "__main__":
    share_url = "https://v.douyin.com/c-NqAPROei8/"
    DouYinDownload().parsec(share_url)
