function cmsEnumInputOnchange(el) {
  /** @type Element */
  const data = el.parentElement.parentElement.querySelector(".cms-enum-data");
  const idx = Array.prototype.indexOf.call(el.parentElement.children, el) / 2;
  for (let i = 0; i < idx; i++) {
    const c = data.children[i];
    c.classList.add("cms-enum-hidden", "cms-enum-hidden-left");
    c.classList.remove("cms-enum-hidden-right");
    c.disabled = true;
  }
  const c = data.children[idx];
  c.classList.remove("cms-enum-hidden", "cms-enum-hidden-right", "cms-enum-hidden-left");
  c.disabled = false;
  for (let i = idx + 1; i < data.childElementCount; i++) {
    const c = data.children[i];
    c.classList.add("cms-enum-hidden", "cms-enum-hidden-right");
    c.classList.remove("cms-enum-hidden-left");
    c.disabled = true;
  }
}
