import os
import re
import json
import requests
from dotenv import load_dotenv

load_dotenv()

SSID = os.getenv("SSID")
PROXY = os.getenv("PROXY")
CLIENT_VERSION_OVERRIDE = os.getenv("CLIENT_VERSION", "").strip() or None

SHARD = "ap"

AUTH_COOKIES_URL = "https://auth.riotgames.com/api/v1/authorization"
AUTH_REAUTH_URL = "https://auth.riotgames.com/authorize"
ENTITLEMENTS_URL = "https://entitlements.auth.riotgames.com/api/token/v1"
USERINFO_URL = "https://auth.riotgames.com/userinfo"

AUTH_BODY = {
    "client_id": "play-valorant-web-prod",
    "nonce": "1",
    "redirect_uri": "https://playvalorant.com/opt_in",
    "response_type": "token id_token",
    "scope": "account openid",
}

CLIENT_PLATFORM = "ew0KCSJwbGF0Zm9ybVR5cGUiOiAiUEMiLA0KCSJwbGF0Zm9ybU9TIjogIldpbmRvd3MiLA0KCSJwbGF0Zm9ybU9TVmVyc2lvbiI6ICIxMC4wLjE5MDQyLjEuMjU2LjY0Yml0IiwNCgkicGxhdGZvcm1DaGlwc2V0IjogIlVua25vd24iDQp9"


def get_session():
    session = requests.Session()
    if PROXY:
        session.proxies = {"https": PROXY, "http": PROXY}
    session.headers.update({
        "User-Agent": "RiotClient/70.0.0.4888690.4873386 rso-auth (Windows;10;;Professional, x64)",
    })
    return session


def authenticate(session):
    session.post(AUTH_COOKIES_URL, headers={"Content-Type": "application/json"}, json=AUTH_BODY)
    session.cookies.set("ssid", SSID, domain="auth.riotgames.com")
    resp = session.get(AUTH_REAUTH_URL, params=AUTH_BODY, allow_redirects=False)
    if resp.status_code not in (301, 303):
        raise Exception(f"認証失敗 (status: {resp.status_code})")
    match = re.search(r"access_token=([^&]+)", resp.headers.get("Location", ""))
    if not match:
        raise Exception("アクセストークン取得失敗")
    return match.group(1)


def get_entitlements_token(session, access_token):
    resp = session.post(
        ENTITLEMENTS_URL,
        headers={"Authorization": f"Bearer {access_token}", "Content-Type": "application/json"},
        json={},
    )
    resp.raise_for_status()
    return resp.json()["entitlements_token"]


def get_puuid(session, access_token):
    resp = session.get(USERINFO_URL, headers={"Authorization": f"Bearer {access_token}"})
    resp.raise_for_status()
    return resp.json()["sub"]


def get_client_version(session):
    if CLIENT_VERSION_OVERRIDE:
        return CLIENT_VERSION_OVERRIDE
    try:
        resp = session.get("https://valorant-api.com/v1/version", timeout=5)
        if resp.status_code == 200:
            version = resp.json().get("data", {}).get("riotClientVersion")
            if version:
                return version
    except Exception:
        pass
    return "release-12.02-shipping-9-4226954"


def get_storefront(session, access_token, entitlements_token, puuid, client_version):
    headers = {
        "Authorization": f"Bearer {access_token}",
        "X-Riot-Entitlements-JWT": entitlements_token,
        "X-Riot-ClientPlatform": CLIENT_PLATFORM,
        "X-Riot-ClientVersion": client_version,
    }
    for method, url in [
        ("GET", f"https://pd.{SHARD}.a.pvp.net/store/v2/storefront/{puuid}"),
        ("POST", f"https://pd.{SHARD}.a.pvp.net/store/v3/storefront/{puuid}"),
        ("GET", f"https://pd.{SHARD}.a.pvp.net/store/v1/storefront/{puuid}"),
    ]:
        resp = session.request(method, url, headers=headers, json={} if method == "POST" else None)
        if resp.status_code == 200:
            return resp.json()
    raise Exception("ストアフロント取得失敗")


def main():
    if not SSID:
        print("エラー: .env に SSID が設定されていません。")
        return

    try:
        session = get_session()
        access_token = authenticate(session)
        entitlements_token = get_entitlements_token(session, access_token)
        puuid = get_puuid(session, access_token)
        client_version = get_client_version(session)
        storefront = get_storefront(session, access_token, entitlements_token, puuid, client_version)
        print(json.dumps(storefront, indent=2, ensure_ascii=False))
    except requests.exceptions.ProxyError:
        print("エラー: プロキシ接続失敗。PROXY設定を確認してください。")
    except Exception as e:
        print(f"エラー: {e}")


if __name__ == "__main__":
    main()