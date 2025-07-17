import requests
import json
import re

videoUrl = "https://www.kuaishou.com/short-video/3xv6gxfhup6c8mm?cc=share_copylink&followRefer=151&shareMethod=TOKEN&docId=9&kpn=KUAISHOU&subBiz=BROWSE_SLIDE_PHOTO&photoId=3xv6gxfhup6c8mm&shareId=18481413877959&shareToken=X-9VNuEG3zVor1lz&shareResourceType=PHOTO_OTHER&userId=3x52iwuwpxbr7qk&shareType=1&et=1_i%252F2008597717177534465_bs6502&shareMode=APP&efid=0&originShareId=18481413877959&appType=21&shareObjectId=5201376260098820311&shareUrlOpened=0&timestamp=1752739957588&utm_source=app_share&utm_medium=app_share&utm_campaign=app_share&location=app_share"
# https://www.kuaishou.com/video/3xbszt6yravw739?authorId=3x45xripnn3tq5a&tabId=1&area=recommendxxfilm
photoId = re.findall(r"https://www.kuaishou.com/video/(.*)\?.*", videoUrl)[0]
try:
    webPageArea = re.findall(r".*area=(.*)", videoUrl)[0]
except Exception as e:
    print(e)
    webPageArea = ""

url = "https://www.kuaishou.com/graphql"
print(photoId, webPageArea)

headers = {
    "content-type": "application/json",
    "Cookie": "kpf=PC_WEB; kpn=KUAISHOU_VISION; clientid=3; did=web_d5468278a1e92934b3751f249005ffd3; client_key=65890b29; userId=1002026148; kuaishou.server.web_st=ChZrdWFpc2hvdS5zZXJ2ZXIud2ViLnN0EqABb3k7H6FPn5rPTxi_U90A1EyivbCG1lWIL1_rV6NDVpXzhBc1jBu7ym59g3fUExHe5ZNBf8YKAXnF2IL0cRoE0cCKrTTndeqWCsDxF5FFXwEjJVAWhZTi87lyyhTvvOBDBgG7AZ03thuTfr_QXmvbueTi9Yd_RBKwIWhWZ9cY88tn8OqN_4iX9l7mHh7PvK7eCnL3BXSM0JABpysi1936rxoS1KQylfZfbCBEuMI0IcjfqenKIiBZWJZqfmVkyM3JqsKIBJh0A-rz8bGOJ_hOfif7PIQInSgFMAE; kuaishou.server.web_ph=4185d09841f87d0bcad53e5c0029de4c6304",
    "Host": "www.kuaishou.com",
    "Origin": "https://www.kuaishou.com",
    "Referer": "https://www.kuaishou.com/search/video?searchKey=%E6%80%A7%E6%84%9F",
    "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/92.0.4515.159 Safari/537.36",
}
data = json.dumps(
    {
        "operationName": "visionVideoDetail",
        "variables": {
            "photoId": "%s" % (photoId),
            "page": "detail",
            "webPageArea": "{}".format(webPageArea),
        },
        "query": "query visionVideoDetail($photoId: String, $type: String, $page: String, $webPageArea: String) {\n  "
        "visionVideoDetail(photoId: $photoId, type: $type, page: $page, webPageArea: $webPageArea) {\n    "
        "status\n    type\n    author {\n      id\n      name\n      following\n      headerUrl\n      "
        "__typename\n    }\n    photo {\n      id\n      duration\n      caption\n      likeCount\n      "
        "realLikeCount\n      coverUrl\n      photoUrl\n      liked\n      timestamp\n      expTag\n      "
        "llsid\n      viewCount\n      videoRatio\n      stereoType\n      croppedPhotoUrl\n      manifest {"
        "\n        mediaType\n        businessType\n        version\n        adaptationSet {\n          id\n "
        "         duration\n          representation {\n            id\n            defaultSelect\n          "
        "  backupUrl\n            codecs\n            url\n            height\n            width\n           "
        " avgBitrate\n            maxBitrate\n            m3u8Slice\n            qualityType\n            "
        "qualityLabel\n            frameRate\n            featureP2sp\n            hidden\n            "
        "disableAdaptive\n            __typename\n          }\n          __typename\n        }\n        "
        "__typename\n      }\n      __typename\n    }\n    tags {\n      type\n      name\n      "
        "__typename\n    }\n    commentLimit {\n      canAddComment\n      __typename\n    }\n    llsid\n    "
        "danmakuSwitch\n    __typename\n  }\n}\n",
    }
)  # 请求的data数据，json类型

rsp = requests.post(url=url, headers=headers, data=data)
infos = rsp.json()
print("响应的状态码为：", rsp.status_code)
print("响应：", infos)
info = infos["data"]["visionVideoDetail"]["photo"]
videoName = info["caption"]
for str_i in ["?", "、", "╲", "/", "*", "“", "”", "<", ">", "|"]:
    videoName = videoName.replace(str_i, "")  # 文件重命名
print("视频的标题：", videoName)
downloadUrl = info["photoUrl"]
print("视频的下载链接为：", downloadUrl)
rsp2 = requests.get(url=downloadUrl, headers=headers)
with open(file="{}.mp4".format(videoName), mode="wb") as f:
    f.write(rsp2.content)
