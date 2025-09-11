/**
 * evaluate the string attribute `onmount` on all children. Stop recusion if `onmount` evaluates to `true`.
 * @param {HTMLElement} e
 */
async function callOnMountRecursive(e) {
  const AsyncFunction = Object.getPrototypeOf(async function () {}).constructor;
  for (const c of e.children) {
    try {
      const a = c.getAttribute("onmount");
      if (!a || !(await new AsyncFunction(a).call(c))) {
        await callOnMountRecursive(c);
      }
    } catch (err) {
      console.error(err, e, c);
    }
  }
}
