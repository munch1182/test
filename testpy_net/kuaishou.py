import re
from typing import Dict

import execjs
import requests


class KuaiShouDownload:
    def __init__(self):
        self.headers = {
            "Connection": "keep-alive",
            "Pragma": "no-cache",
            "Cache-Control": "no-cache",
            "sec-ch-ua": '"Google Chrome";v="95", "Chromium";v="95", ";Not A Brand";v="99"',
            "accept-language": "zh-CN,zh;q=0.9,ru;q=0.8",
            "sec-ch-ua-mobile": "?0",
            "User-Agent": "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/95.0.4638.54 Safari/537.36",
            "content-type": "application/json",
            "accept": "*/*",
            "sec-ch-ua-platform": '"Linux"',
            "Origin": "https://www.kuaishou.com",
            "Sec-Fetch-Site": "same-origin",
            "Sec-Fetch-Mode": "cors",
            "Sec-Fetch-Dest": "empty",
            "Referer": "https://www.kuaishou.com/search/video?searchKey=%E4%BD%A0%E5%AF%B9%E7%88%B1",
        }

    def download(self, shareUrl):
        session = requests.session()
        req = session.head(shareUrl, headers=self.headers, allow_redirects=True)
        print(f"req url: {req.url}")

        vedioId = "3x6eet6wh8dyjmi"
        data = self.data(vedioId)

        print(f"data: {data}")
        url = "https://www.kuaishou.com/graphql"
        resp = requests.post(url=url, json=data, headers=self.headers)
        print(resp.status_code)
        json = resp.json()
        did = resp.cookies.get("did")
        print(json)
        if json["data"]["result"] == 400002:
            print("需要验证码")
            self.verify(json["data"], did)
        else:
            print("不需要验证码")

    def data(self, photo_id):
        return {
            "operationName": "visionProfilePhotoList",
            "query": "query visionProfilePhotoList($pcursor: String, $userId: String, $page: String, $webPageArea: String) {\n  visionProfilePhotoList(pcursor: $pcursor, userId: $userId, page: $page, webPageArea: $webPageArea) {\n    result\n    llsid\n    webPageArea\n    feeds {\n      type\n      author {\n        id\n        name\n        following\n        headerUrl\n        headerUrls {\n          cdn\n          url\n          __typename\n        }\n        __typename\n      }\n      tags {\n        type\n        name\n        __typename\n      }\n      photo {\n        id\n        duration\n        caption\n        likeCount\n        realLikeCount\n        coverUrl\n        coverUrls {\n          cdn\n          url\n          __typename\n        }\n        photoUrls {\n          cdn\n          url\n          __typename\n        }\n        photoUrl\n        liked\n        timestamp\n        expTag\n        animatedCoverUrl\n        stereoType\n        videoRatio\n        profileUserTopPhoto\n        __typename\n      }\n      canAddComment\n      currentPcursor\n      llsid\n      status\n      __typename\n    }\n    hostName\n    pcursor\n    __typename\n  }\n}\n",
            "variables": {
                "userId": "3xhv7zhkfr3rqag",
                "pcursor": photo_id,
                "page": "detail",
                "webPageArea": "profilexxnull",
            },
        }

    def verify(self, data, did):
        url = "https://api.zt.kuaishou.com/rest/zt/captcha/sliding/config"
        captchaSession = re.findall("captchaSession=(.*)&type", data["url"])[0]
        print(f"captchaSession: {captchaSession}")
        data = {"captchaSession": captchaSession}
        headers = self.headers
        headers["content-type"] = "application/x-www-form-urlencoded"

        resp = requests.post(url=url, data=data, headers=self.headers)

        print(resp.status_code)
        print(resp.json())

        captcha_data = {
            "captchaSn": captchaSession,  # '上面config里的captchaSn'
            "bgDisWidth": 316,  # 可以写死，根据config里的值*0.46，四舍五入得来
            "bgDisHeight": 184,  # 同上，可以写死
            "cutDisWidth": 56,  # 同上，可以写死
            "cutDisHeight": 56,  # 同上，可以写死
            "relativeX": self.distance,  # '需要自己识别滑块的距离',
            "relativeY": self.disY,  # '同上config里的值*0.46，四舍五入，但不可以写死',
            "trajectory": self.trajectory,
            # "滑块拖动轨迹",
            "gpuInfo": '{"glRenderer":"WebKit WebGL","glVendor":"WebKit","unmaskRenderer":"ANGLE (Intel, Intel(R) HD Graphics 630 Direct3D11 vs_5_0 ps_5_0, D3D11-31.0.101.2111)","unmaskVendor":"Google Inc. (Intel)"}',
            # '显卡信息，可以写死',
            "captchaExtraParam": '{"ua":"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/95.0.4638.69 Safari/537.36","userAgent":"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/95.0.4638.69 Safari/537.36","timeZone":"UTC+8","language":"zh-CN","cpuCoreCnt":"4","platform":"Win32","riskBrowser":"false","webDriver":"false","exactRiskBrowser":"false","webDriverDeep":"false","exactRiskBrowser2":"false","webDriverDeep2":"false","battery":"1","plugins":"170b76c0d6cbaed3cb42746b06cae5eae","resolution":"1920x1080","pixelDepth":"24","colorDepth":"24","canvasGraphFingerPrint":"184d2a5cb7dd0dbef53dd603e607d58f8","canvasGraph":"184d2a5cb7dd0dbef53dd603e607d58f8","canvasTextFingerPrintEn":"10988b111dee10a3ace1f10536e3a0eee","canvasTextEn":"10988b111dee10a3ace1f10536e3a0eee","canvasTextFingerPrintZh":"13ffb1298cbb06fbd4cbf7a29b627d240","canvasTextZh":"13ffb1298cbb06fbd4cbf7a29b627d240","webglGraphFingerPrint":"14dc51ccd006f78c818999c374ac62402","webglGraph":"14dc51ccd006f78c818999c374ac62402","webglGPUFingerPrint":"1108f3efe4bed369a31b6475af6c38f30","webglGpu":"1108f3efe4bed369a31b6475af6c38f30","cssFontFingerPrintEn":"10a344f5534d5b367655c7f90f04de717","fontListEn":"10a344f5534d5b367655c7f90f04de717","cssFontFingerPrintZh":"16c1334aeae228bca19e18632c8472a52","fontListZh":"16c1334aeae228bca19e18632c8472a52","voiceFingerPrint":"1dd96cac4e826abdbbe261dc4f3a08292","audioTriangle":"1dd96cac4e826abdbbe261dc4f3a08292","nativeFunc":"1973dcbb27a04c3a2ee240d9d2549e105","key1":"web_11896f467df30247503494240be3a7a2","key2":1682662120147,"key3":"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/95.0.4638.69 Safari/537.36","key4":"20030107","key5":"zh-CN","key6":"Gecko","key7":1920,"key8":1080,"key9":1920,"key10":1040,"key11":360,"key12":360,"key13":878,"key14":1358,"key15":"00000111","key16":1,"key17":1,"key18":[],"key19":{},"key20":[],"key21":{},"key22":[],"key23":{},"key24":[],"key25":{},"key26":{"key27":["0,1,3076,267,321,prepare1","1,1,3087,256,314,prepare1","2,1,3092,242,310,prepare1","3,1,3102,230,304,prepare1","4,1,3108,219,300,prepare1","5,1,3116,209,294,prepare1","6,1,3124,199,291,prepare1","7,1,3135,191,288,prepare1","8,1,3158,173,281,prepare1","9,1,3164,169,278,prepare1","10,3,3919,44,232","11,1,3940,44,233,prepare2","12,1,3956,45,234,prepare2","13,1,3964,46,235,prepare2","14,1,3972,47,235,prepare2","15,1,3980,51,237,prepare2","16,1,3988,54,237,prepare2","17,1,3996,58,239,prepare2","18,1,4004,61,239,prepare2","19,1,4012,65,239,prepare2","20,1,4020,68,239,prepare2","21,4,4484,193,234","22,2,4688,193,234,prepare3","23,1,4689,262,321,prepare3"],"key28":[],"key29":[],"key30":[],"key31":{"prepare1":"9,1,3164,169,278","prepare2":"20,1,4020,68,239","prepare3":"23,1,4689,262,321"},"key32":{},"key33":{},"key34":{}},"key35":"7ebc4735321e3b0c225c1e489d2adb1b","key36":"f22a94013fc94e90e2af2798023a1985","key37":1,"key38":"not support","key39":4}',
            # '浏览器信息和指纹什么的'
        }
        # 加载json数据
        js = self.loadjs(data["jsSdkuRL"])

        js.call("get_data", captcha_data)

    def loadjs(self, jsSdkuRL):
        with open("./encrypt1.js") as f:
            js = f.read().decode()
            return execjs.compile(js)


if __name__ == "__main__":
    url = "https://v.kuaishou.com/Jl8Z7K8b"
    KuaiShouDownload().download(url)
