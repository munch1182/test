pub(crate) fn setup_script() -> &'static str {
    r#"
/**
 * 为窗口提供自定义拖动、关闭和最小化功能
 * - 拖动：元素带有 data-decoration 属性时，可拖动窗口（阻止默认拖拽效果）
 * - 关闭：元素带有 data-command="close" 时，点击发送关闭指令
 * - 最小化：元素带有 data-command="minimize" 时，点击发送最小化指令
 */
(function() {
  // ----- 1. 拖动相关 -----
  function initDraggable() {
    document.querySelectorAll('[data-decoration]').forEach(setDraggable);
  }

  function setDraggable(el) {
    if (!el.draggable) {
      el.draggable = true;
    }
  }

  document.addEventListener('dragstart', (event) => {
    const target = event.target;
    if (target.nodeType === Node.ELEMENT_NODE && target.hasAttribute('data-decoration')) {
      event.preventDefault(); // 禁用浏览器默认拖拽效果
      console.log('拖动开始，元素：', target);
      window.ipc.postMessage("DragStart|");
    }
  });

  // ----- 2. 窗口命令（关闭/最小化） -----
  document.addEventListener('click', (event) => {
    const target = event.target.closest('[data-command]'); // 获取最近的命令元素
    if (!target) return;

    const command = target.getAttribute('data-command');
    event.preventDefault(); // 阻止可能的默认行为（如表单提交）

    switch (command) {
      case 'close':
        window.ipc.postMessage("Close|");
        break;
      case 'minimize':
        window.ipc.postMessage("Minimize|");
        break;
      default:
        console.warn('未知窗口命令：', command);
    }
  });

  // ----- 4. 启动 -----
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
      initDraggable();
    });
  } else {
    initDraggable();
  }
})();
    "#
}