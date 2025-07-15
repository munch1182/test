import re
import urllib

import requests
import execjs


def ttwid():
    data = '{"region":"cn","aid":1768,"needFid":false,"service":"www.ixigua.com","migrate_info":{"ticket":"","source":"node"},"cbUrlProtocol":"https","union":true}'
    res = requests.request(
        "POST", "https://ttwid.bytedance.com/ttwid/union/register/", data=data
    )
    print(res.headers)
    cookie = res.headers["Set-Cookie"]
    return re.findall("ttwid=(.*?);", cookie)[0]


def abogus(params, agent):
    with open("./abogus.js", encoding="utf-8") as f:
        js = execjs.compile(f.read())
        return js.call(
            "sign_datail",
            params,
            agent,
        )
