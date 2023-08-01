"use strict";

function explain_assertions() {
    let input = document.getElementById("Input").value;
    let format_div = document.getElementById("Format");
    let output_div = document.getElementById("Output");
    let explanation_div = document.getElementById("Explanation");
    removeAllChildElements(format_div);
    removeAllChildElements(output_div);
    removeAllChildElements(explanation_div);
    let parsed_input = null;
    try {
        parsed_input=JSON.parse(input);
    } catch (e) {
        add(output_div,"p","error").innerText="Error : input is not JSON";
        return;
    }
    const contests = interpret_input_formats(parsed_input);
    if (contests && contests.format) {
        add(format_div,"p").innerText="Format : "+contests.format;
        for (const contest of contests.contests) {
            describe_raire_result(output_div,explanation_div,contest);
        }
    } else {
        add(output_div,"p","error").innerText="Error : could not establish input format";
    }
}


function interpret_input_formats(input) {
    if (input.hasOwnProperty("metadata") && input.hasOwnProperty("solution")) {
        return {format: "raire-rs", contests: [input]};
    } else {
        let Michelle = convert_from_Michelle_format(input);
        if (Michelle) return { format: "Michelle Blom RAIRE", contests: Michelle }
        return null;
    }
}

function convert_from_Michelle_format(input) {
    if (!input.hasOwnProperty("parameters")) return null;
    if (!Array.isArray(input.audits)) return null;
    let contests = [];
    for (const audit of input.audits) {
        const candidates = [audit.winner].concat(audit.eliminated);
        const metadata = { contest : audit.contest || "Unnamed contest", candidates : candidates };
        const candidate_id_of_name = {}; // inverse of candidates
        const num_candidates = candidates.length;
        for (let i=0;i<num_candidates;i++) candidate_id_of_name[candidates[i]]=i;
        const assertions = [];
        for (const assertion of audit.assertions) {
            const out = {
                "winner" : candidate_id_of_name[assertion.winner],
                "loser" : candidate_id_of_name[assertion.loser],
            }
            if (assertion.assertion_type==="IRV_ELIMINATION") {
                out.type = "NEN";
                out.continuing = [];
                for (let i=0;i<num_candidates;i++) if (!assertion.already_eliminated.includes(candidates[i])) out.continuing.push(i);
            } else if (assertion.assertion_type==="WINNER_ONLY") {
                out.type = "NEB";
            } else return null;
            assertions.push({assertion: out});
        }
        contests.push({
            metadata : metadata,
            solution : {
                Ok : {
                    assertions : assertions,
                    winner:0,
                    num_candidates:num_candidates
                }
            }
        })
    }
    return contests;
}