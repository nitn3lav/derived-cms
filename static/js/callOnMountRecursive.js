/**
 * evaluate the string attribute `onmount` on all children. Stop recusion if `onmount` evaluates to `true`.
 * @param {HTMLElement} e
 */
function callOnMountRecursive(e) {
  for (const c of e.children) {
    try {
      const a = c.getAttribute("onmount");
      if (!a || !new Function(a).call(c)) {
        callOnMountRecursive(c);
      }
    } catch (err) {
      console.error(err, e, c);
    }
  }
}
