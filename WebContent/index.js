"use strict";

let last_computed_output = null; // kept as a global variable so that checkbox changing doesn't have to recall execute_raire().

function execute_raire() {
    let input = document.getElementById("Input").value;
    let output_div = document.getElementById("Output");
    let explanation_div = document.getElementById("Explanation");
    removeAllChildElements(output_div);
    removeAllChildElements(explanation_div);
    let parsed_input = null;
    try { parsed_input=JSON.parse(input) } catch (e) {
        add(output_div,"p","error").innerText="Error : input is not JSON";
        return;
    }
    add(output_div,"p","computing").innerText="Computing...";
    add(explanation_div,"p","computing").innerText="Computing...";
    function failure(message) {
        removeAllChildElements(output_div);
        add(output_div,"p","error").innerText="Error : "+message;
    }
    function success(data) {
        // console.log(data);
        last_computed_output=data;
        explain_assertions();
    }
    getWebJSON("raire",success,failure,input,"application/json");
}

function explain_assertions() {
    let output_div = document.getElementById("Output");
    let explanation_div = document.getElementById("Explanation");
    removeAllChildElements(output_div);
    removeAllChildElements(explanation_div);
    describe_raire_result(output_div,explanation_div,last_computed_output);
}

function load_example(url) {
    function failure(message) {
        alert("Could not load "+url+" sorry. Message :"+message);
    }
    function success(text) {
        document.getElementById("Input").value=text;
        execute_raire();
    }
    getWebJSON(url,success,failure,null,null,"text");
}

function make_examples() {
    function make_example(name,url,where) {
        const dom = document.getElementById(where);
        const a = add(dom,"a","example");
        a.href = url;
        a.textContent = name;
        a.onclick = function () { load_example(url); return false; }
    }
    // make "a guide to RAIRE" examples
    for (const name of ["guide","NEB_assertions","one_candidate_dominates","two_leading_candidates","why_not_audit_every_step"]) {
        make_example(name.replace("_"," "),"example_input/a_guide_to_RAIRE_eg_"+name+".json","EgGuideToRaire");
    }
}

window.onload = function () {
    make_examples();
    document.getElementById('InputFile').addEventListener('change', function() {
        const filereader = new FileReader();
        filereader.onload = () => {
            document.getElementById("Input").value = filereader.result;
        };
        filereader.readAsText(this.files[0]);
    });
    document.getElementById("ExpandAtStart").addEventListener('change',explain_assertions);
    document.getElementById("DrawAsText").addEventListener('change',explain_assertions);
    document.getElementById("HideWinner").addEventListener('change',explain_assertions);
}