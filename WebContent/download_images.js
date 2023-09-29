"use strict";
/* Copyright 2021-2023 Andrew Conway. */

function triggerDownload (imgURI,filenamebase,extension) {
    const evt = new MouseEvent('click', {
        view: window,
        bubbles: false,
        cancelable: true
    });

    const a = document.createElement('a');
    a.setAttribute('download',filenamebase+(extension||".svg"));
    a.setAttribute('href', imgURI);
    a.setAttribute('target', '_blank');

    a.dispatchEvent(evt);
}

function findCSSStyleSheet(name) {
    //console.log(document);
    for (const css of document.getElementsByTagName("link")) {
        if (css.sheet && css.sheet.href.endsWith(name) && css.sheet.cssRules) {
            //console.log(css.sheet.cssRules);
            let res = "";
            for (const rule of css.sheet.cssRules) {
                res+=rule.cssText+"\n";
            }
            //console.log(res);
            return res;
        }
    }
    console.log("Could not find style sheet.")
    return "";
}

/**
 * An array of the images that should be saved in saveAllImages
 * @type {{svg:Element,name:string}[]}
 */
let allImages = [];

function saveAllImages() {
    const styleText = findCSSStyleSheet("common.css");
    for (const image of allImages) {
        const svg = image.svg;
        const firstChild = svg.childNodes[0];
        const style = document.createElementNS(svgNS,"style");
        svg.setAttribute("viewBox", "0 0 "+svg.clientWidth+" "+svg.clientHeight);
        style.type = 'text/css';
        style.appendChild(document.createTextNode(styleText));
        svg.insertBefore(style,firstChild);
        const data = (new XMLSerializer()).serializeToString(svg);
        const DOMURL = window.URL || window.webkitURL || window;
        const svgBlob = new Blob([data], {type: 'image/svg+xml;charset=utf-8'});
        const url = DOMURL.createObjectURL(svgBlob);
        triggerDownload(url,image.name,".svg");
        style.remove();
        svg.removeAttribute("viewBox");
    }
    alert("Check your downloads folder for downloaded images.")
}
