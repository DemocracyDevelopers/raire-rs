"use strict";


function explain_assertions() {
    checkOptionVisibility();
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
        let ShangriLa = convert_from_ShangriLa_log_format(input);
        if (ShangriLa) return { format: "ShangriLa log", contests: ShangriLa }
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
        const assertions = parseMichelleAssertions(audit.assertions,candidates);
        if (assertions===null) return null;
        contests.push({
            metadata : metadata,
            solution : {
                Ok : {
                    assertions : assertions,
                    winner:0,
                    num_candidates:candidates.length
                }
            }
        })
    }
    return contests;
}

function parseMichelleAssertions(audit_assertions,candidates) {
    const candidate_id_of_name = {}; // inverse of candidates
    const num_candidates = candidates.length;
    for (let i=0;i<num_candidates;i++) candidate_id_of_name[candidates[i]]=i;
    const assertions = [];
    for (const assertion of audit_assertions) {
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
    return assertions;
}

function convert_from_ShangriLa_log_format(input) {
    if (!input.hasOwnProperty("seed")) return null;
    if (!input.hasOwnProperty("contests")) return null;
    let contests = [];
    for (const contest_id of Object.getOwnPropertyNames(input.contests)) {
        const contest = input.contests[contest_id];
        if (contest.n_winners!==1) continue;
        if (!Array.isArray(contest.reported_winners)) continue;
        if (contest.reported_winners.length!==1) continue;
        if (contest.choice_function!=="IRV") continue;
        const winner = contest.candidates.indexOf(contest.reported_winners[0]);
        if (winner=== -1) continue;
        const assertions=parseMichelleAssertions(contest.assertion_json,contest.candidates);
        if (assertions===null) return null;
        contests.push({
            metadata : { contest : contest_id, candidates: contest.candidates },
            solution : {
                Ok : {
                    assertions : assertions,
                    winner: winner,
                    num_candidates:contest.candidates.length
                }
            }
        });
    }
    return contests;
}

function load_example(url) {
    function failure(message) {
        alert("Could not load "+url+" sorry. Message :"+message);
    }
    function success(text) {
        document.getElementById("Input").value=text;
        explain_assertions();
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
        make_example(name.replace("_"," "),"example_assertions/a_guide_to_RAIRE_eg_"+name+".json","EgGuideToRaire");
    }
    make_example("San Francisco IRV RLA pilot 2019","https://raw.githubusercontent.com/DemocracyDevelopers/SHANGRLA/refs/heads/main/shangrla/Examples/Data/SFDA2019/SF2019Nov8Assertions.json","MichelleExamples");
    make_example("San Francisco IRV RLA pilot 2019","SHANGRLA_SF2019_log_with_write_in.json","SHANGRLAExamples"); // candiaate 45 added to "https://github.com/DemocracyDevelopers/SHANGRLA/blob/main/shangrla/Examples/log.json"
}

window.onload = function () {
    make_examples();
    checkOptionVisibility();
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
    document.getElementById("ShowEffectOfEachAssertionSeparately").addEventListener('change',explain_assertions);
    document.getElementById("preventTextOverlapping").addEventListener('change',explain_assertions);
    document.getElementById("showAssertionIndex").addEventListener('change',explain_assertions);
    document.getElementById("showAssertionText").addEventListener('change',explain_assertions);
    document.getElementById("splitGreaterThanLines").addEventListener('change',explain_assertions);
}