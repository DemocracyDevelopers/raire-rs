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
        function candidate_name(id) {
            if (data.metadata && Array.isArray(data.metadata.candidates)) {
                let name = data.metadata.candidates[id];
                if (name) { return name; }
            }
            return "Candidate "+id;
        }
        function candidate_name_list(ids) {
            return ids.map(candidate_name).join(",")
        }
        if (data.solution && data.solution.Ok) {
            add(output_div,"h3","Assertions").innerText="Assertions - difficulty = "+data.solution.Ok.difficulty;
            for (const av of data.solution.Ok.assertions) {
                let adiv = add(output_div,"div");
                add(adiv,"span","difficulty_start").innerText=""+av.difficulty;
                const a = av.assertion;
                const adesc = add(adiv,"span");
                if (a["type"] === "NEN") {
                    adesc.innerText="NEN: "+candidate_name(a.winner)+" > "+candidate_name(a.loser)+" if only {"+candidate_name_list(a.continuing)+"} remain";
                } else if (a["type"] === "NEB") {
                    adesc.innerText=candidate_name(a.winner)+" NEB "+candidate_name(a.loser);
                } else {
                    adesc.innerText="Unknown assertion type"
                }
            }
            let candidate_names = data.metadata && data.metadata.candidates;
            if (!(Array.isArray(candidate_names) && candidate_names.length===parsed_input.num_candidates)) {
                candidate_names = [];
                for (let i=0;i<parsed_input.num_candidates;i++) { candidate_names.push(candidate_name(i)); }
            }
            explain(explanation_div,data.solution.Ok.assertions.map(a=>a.assertion),candidate_names,document.getElementById("ExpandAtStart").checked,document.getElementById("DrawAsText").checked);
        } else if (data.solution.Err) {
            let err = data.solution.Err;
            if (err==="Timeout") {
                add(output_div,"p","error").innerText="Timeout - the problem seemed to take too long";
            } else if (Array.isArray(err.CouldNotRuleOut)) {
                add(output_div,"p","error").innerText="Impossible to audit. Could not rule out the following elimination order:";
                for (let i=0;i<err.CouldNotRuleOut.length;i++) {
                    add(output_div,"p","candidate_name").innerText=candidate_name(err.CouldNotRuleOut[i])+(i===0?" (First elimimated)":"")+(i===err.CouldNotRuleOut.length-1?" (Winner)":"");
                }
            } else if (Array.isArray(err.TiedWinners)) {
                add(output_div,"p","error").innerText="Audit not possible as "+candidate_name_list(err.TiedWinners)+" are tied IRV winners and a one vote difference would change the outcome.";
            } else if (Array.isArray(err.WrongWinner)) {
                add(output_div,"p","error").innerText="The votes are not consistent with the provided winner. Perhaps "+candidate_name_list(err.WrongWinner)+"?";
            } else {
                add(output_div,"p","error").innerText="Error : "+JSON.stringify(err);
            }
        } else {
            add(output_div,"p","error").innerText="Output is wrong format";
        }
    }
    getWebJSON("raire",success,failure,input,"application/json");
}