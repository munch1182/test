//! 内部常量定义

/// 前端桥接对象挂载的全局变量名（内部使用，不对外暴露）
pub const BRIDGE_INTERNAL: &str = "__bridge";

/// 前端桥接对象公开的 API 名称（供应用层使用，需保持稳定）
pub const BRIDGE_PUBLIC: &str = "bridge";

/// 后端回调前端响应处理函数的方法名（挂载在内部桥接对象上）
pub const BRIDGE_HANDLER_METHOD: &str = "_handleResponse";

/// 完整的后端调用表达式，用于 evaluate_script
/// 格式：window.__bridge._handleResponse(response)
pub fn bridge_handler_call(response_json: &str) -> String {
    format!("window.{}.{}({});", BRIDGE_INTERNAL, BRIDGE_HANDLER_METHOD, response_json)
}

pub(crate) fn setup_script() -> String {
    format!(r#"
/**
 * 前端 IPC 桥接模块
 * 
 * 提供基于 Promise 的 RPC 风格调用，通过 window.{public}.send(command, payload) 发送请求，
 * 后端处理后通过 window.{internal}.{handler} 将结果传回，并 resolve 对应的 Promise。
 * 
 * 底层使用 wry 注入的 window.ipc.postMessage 发送 JSON 格式消息，消息格式为：
 *   {{ id: number, command: string, payload: any }}
 * 
 * 响应格式：
 *   {{ id: number, payload: any }}   （成功时 payload 为任意值）
 *   若 payload 包含 error 字段，则 Promise 会 reject。
 * 
 * 同时处理窗口系统命令（拖动、关闭、最小化），这些命令也通过 {public}.send 发送，
 * 但无需等待响应。
 */

(function() {{
  // ----- 1. 内部状态和工具函数 -----
  const BRIDGE = {{
    _nextId: 1,                 // 自增请求 ID
    _callbacks: new Map(),       // 存储等待中的 Promise 回调 {{ resolve, reject }}

    /**
     * 处理后端返回的响应
     * @param {{Object}} response - 响应对象，包含 id 和 payload
     */
    {handler}: function(response) {{
      const cb = this._callbacks.get(response.id);
      if (cb) {{
        this._callbacks.delete(response.id);
        // 如果 payload 是对象且包含 error 字段，则 reject; 否则 resolve
        if (response.payload && typeof response.payload === 'object' && response.payload.error) {{
          cb.reject(new Error(response.payload.error));
        }} else {{
          cb.resolve(response.payload);
        }}
      }} else {{
        console.warn('未找到请求 ID 对应的回调:', response.id);
      }}
    }},

    /**
     * 发送命令到后端
     * @param {{string}} command - 命令名称
     * @param {{any}} payload - 任意 JSON 可序列化的数据
     * @returns {{Promise<any>}} 后端返回的 payload
     */
    send: function(command, payload) {{
      const id = this._nextId++;
      return new Promise((resolve, reject) => {{
        this._callbacks.set(id, {{ resolve, reject }});
        const msg = {{ id, command, payload }};
        // 使用 wry 底层提供的 postMessage 发送 JSON 字符串
        window.ipc.postMessage(JSON.stringify(msg));
      }});
    }}
  }};

  // 将内部对象挂载到全局（用于后端回调），同时暴露简化版的公共 API
  window.{internal} = BRIDGE;
  window.{public} = {{
    send: window.{internal}.send.bind(window.{internal})
  }};

  // ----- 2. 窗口控制功能（拖动、关闭、最小化）-----
  /**
   * 初始化可拖动元素：为带有 data-decoration 属性的元素设置 draggable=true
   */
  function initDraggable() {{
    document.querySelectorAll('[data-decoration]').forEach(el => {{
      if (!el.draggable) el.draggable = true;
    }});
  }}

  // 监听拖动开始事件，发送 DragStart 命令
  document.addEventListener('dragstart', (event) => {{
    const target = event.target;
    if (target.nodeType === Node.ELEMENT_NODE && target.hasAttribute('data-decoration')) {{
      event.preventDefault(); // 阻止浏览器默认拖拽行为
      window.{public}.send('DragStart', null).catch(() => {{}}); // 忽略错误
    }}
  }});

  // 监听点击事件，处理关闭/最小化命令
  document.addEventListener('click', (event) => {{
    const target = event.target.closest('[data-command]');
    if (!target) return;
    const command = target.getAttribute('data-command');
    event.preventDefault();

    switch (command) {{
      case 'close':
        window.{public}.send('Close', null).catch(() => {{}});
        break;
      case 'minimize':
        window.{public}.send('Minimize', null).catch(() => {{}});
        break;
      default:
        console.warn('未知窗口命令:', command);
    }}
  }});

  // ----- 3. 启动初始化 -----
  if (document.readyState === 'loading') {{
    document.addEventListener('DOMContentLoaded', initDraggable);
  }} else {{
    initDraggable();
  }}
}})();
    "#,
        internal = BRIDGE_INTERNAL,
        public = BRIDGE_PUBLIC,
        handler = BRIDGE_HANDLER_METHOD
    )
}