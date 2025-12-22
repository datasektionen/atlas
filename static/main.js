function switch_lang(target) {
    document.cookie = `Atlas-Lang=${target}; Secure; Path=/`;
    window.location.reload();
}
