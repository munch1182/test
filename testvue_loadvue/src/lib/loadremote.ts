export const opts = {
  async getFile(url: string) {
    const res = await fetch(url);
    const code = await res.text();
    console.log(code);
    return code;
  },
  addStyle(textContent: string) {
    const style = Object.assign(document.createElement("style"), {
      textContent,
    });
    const ref = document.head.getElementsByTagName("style")[0] || null;
    document.head.insertBefore(style, ref);
  },
};
