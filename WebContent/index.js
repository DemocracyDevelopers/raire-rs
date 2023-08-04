"use strict";

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
        console.log(data);
        removeAllChildElements(output_div);
        removeAllChildElements(explanation_div);
        describe_raire_result(output_div,explanation_div,data);
    }
    getWebJSON("raire",success,failure,input,"application/json");
}

window.onload = function () {
    document.getElementById('InputFile').addEventListener('change', function() {
        const filereader = new FileReader();
        filereader.onload = () => {
            document.getElementById("Input").value = filereader.result;
        };
        filereader.readAsText(this.files[0]);
    });
}