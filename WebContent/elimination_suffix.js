"use strict";

// This file contains utility functions to show how a RAIRE assertion
// eliminates potential elimination sequence suffixes.


// enumeration for the effects of an assertion on an elimination order suffix.
const EffectOfAssertionOnEliminationOrderSuffix = {
    Contradiction: Symbol("Contradiction"), // The suffix is ruled out by the assertion, regardless of the rest of the elimination order.
    Ok : Symbol("Ok"), // The suffix is ok as far as the assertion is concerned, regardless of the rest of the elimination order.
    NeedsMoreDetail : Symbol("NeedsMoreDetail") // The suffix is ok as far as the assertion is concerned, regardless of the rest of the elimination order.
}


/**
 * Return an EffectOfAssertionOnEliminationOrderSuffix enum depending on the
 * effect of the assertion on the elimination order suffix.
 * @param {{winner:number,loser:number,continuing:number[],type:string}} assertion The assertion.
 * @param {number[]} elimination_order_suffix A list of candidate indices, being a suffix of the elimination order, the last being the winner.
 * @return {Symbol} a field of EffectOfAssertionOnEliminationOrderSuffix
 */
function assertion_ok_elimination_order_suffix(assertion,elimination_order_suffix) {
    if (assertion.type==="NEN") { // check that the suffix is compatible with the continuing candidates.
        for (let index = Math.max(0,elimination_order_suffix.length-assertion.continuing.length);index<elimination_order_suffix.length;index++) {
            const candidate = elimination_order_suffix[index];
            if (!assertion.continuing.includes(candidate)) { return EffectOfAssertionOnEliminationOrderSuffix.Ok } // the assertion does not say anything about this elimination order or any continuation of it.
        }
        if (elimination_order_suffix.length>=assertion.continuing.length) { // the whole elimination order is all present. The winner cannot be the first eliminated, as self.winner has more votes than self.loser at this point.
            if (elimination_order_suffix[elimination_order_suffix.length-assertion.continuing.length]===assertion.winner) { return EffectOfAssertionOnEliminationOrderSuffix.Contradiction; } else { return EffectOfAssertionOnEliminationOrderSuffix.Ok; }
        }  else {
            if (elimination_order_suffix.includes(assertion.winner)) { // winner wasn't the first eliminated.
                return EffectOfAssertionOnEliminationOrderSuffix.Ok;
            } else {
                return EffectOfAssertionOnEliminationOrderSuffix.NeedsMoreDetail;
            }
        }
    } else { // NEB
        for (let index=elimination_order_suffix.length-1;index>=0;index--) { // look at candidates in reverse order of elimination order, that is winner first.
            const candidate = elimination_order_suffix[index];
            if (candidate===assertion.winner) { return EffectOfAssertionOnEliminationOrderSuffix.Ok } // winner goes better than loser, no problems. If a NEN with incomplete elimination order, either it ie irrelevant => OK or it is good => OK.
            else if (candidate===assertion.loser) { return EffectOfAssertionOnEliminationOrderSuffix.Contradiction; } // loser goes better than winner, no way unless...
        }
        return EffectOfAssertionOnEliminationOrderSuffix.NeedsMoreDetail; // haven't seen the winner or loser yet.
    }
}

/**
 *  given an elimination order suffix,
 *   * let it through if it is allowed,
 *   * block if it is contradicted,
 *   * expand if it is not enough information.
 * @param {{winner:number,loser:number,continuing:number[],type:string}} assertion The assertion.
 * @param {number[]} elimination_order_suffix A list of candidate indices, being a suffix of the elimination order, the last being the winner.
 * @param {number} num_candidates The number of condidates. Candidate numbers are 0..num_candidates-1 inclusive.
 * @param {boolean} just_get_enough_info If true, don't eliminate any contradicted entries, just expand any ambiguous entries.
 * @return {number[][]} a list of possible elimination order suffixes
 */
function assertion_allowed_suffixes(assertion,elimination_order_suffix,num_candidates,just_get_enough_info)  {
    let effect = assertion_ok_elimination_order_suffix(assertion,elimination_order_suffix);
    if (effect===EffectOfAssertionOnEliminationOrderSuffix.Contradiction) {
        if (just_get_enough_info) return [elimination_order_suffix];
        else return [];
    }
    else if (effect===EffectOfAssertionOnEliminationOrderSuffix.Ok) { return [elimination_order_suffix]; }
    else { // must need more information. Extend the suffixes.
        let res = [];
        for (let candidate=0;candidate<num_candidates;candidate++) {
            if (!elimination_order_suffix.includes(candidate)) {
                let v = [candidate].concat(elimination_order_suffix);
                let extras = assertion_allowed_suffixes(assertion,v,num_candidates,just_get_enough_info);
                res.push(...extras);
            }
        }
        return res;
    }
}


/**
 *  Like assertion_allowed_suffixes, except process a list of elimination order suffixes.
 * @param {{winner:number,loser:number,continuing:number[],type:string}} assertion The assertion.
 * @param {number[][]} elimination_order_suffixes A list of elimination order suffixes.
 * @param {number} num_candidates The number of condidates. Candidate numbers are 0..num_candidates-1 inclusive.
 * @param {boolean} just_get_enough_info If true, don't eliminate any contradicted entries, just expand any ambiguous entries.
 * @return {number[][]} a list of possible elimination order suffixes
 */
function assertion_all_allowed_suffixes(assertion,elimination_order_suffixes,num_candidates,just_get_enough_info)  {
    let res = [];
    for (const elimination_order_suffix of elimination_order_suffixes) {
        res.push(...assertion_allowed_suffixes(assertion,elimination_order_suffix,num_candidates,just_get_enough_info));
    }
    return res;
}

/**
 * Get all num_candidates factorial possible orderings
 * @param {number} num_candidates The number of candidates
 * @return {number[][]} a list of all possible full length elimination orders
  */
function all_elimination_orders(num_candidates)  {
    if (num_candidates===0) { return [[]]; }
    let res = [];
    let candidate = num_candidates-1;
    for (const rest of all_elimination_orders(num_candidates-1)) {
        // put candidate in every possible place
        for (let i=0;i<num_candidates;i++) {
            let new_order = rest.slice(0,i).concat([candidate]).concat(rest.slice(i));
            res.push(new_order);
        }
    }
    return res;
}

/**
 * Get all num_candidates single candidate prefixes of an elimination order.
 * @param {number} num_candidates The number of candidates
 * @return {number[][]} a list of all possible single candidate elimination order suffixes
 */
function all_elimination_order_suffixes(num_candidates)  {
    let res = [];
    for (let i=0;i<num_candidates;i++) {
        res.push([i]);
    }
    return res;
}

/**
 * A text description of an assertion
 * @param {{winner:number,loser:number,continuing:number[],type:string}} assertion The assertion.
 * @param {string[]} candidate_names : a list of the candidate names
 * @return {string} a text description of the assertion
 */
function assertion_description(assertion,candidate_names) {
    const basic = candidate_names[assertion.winner]+" beats "+candidate_names[assertion.loser];
    if (assertion.type==="NEB") {
        return basic;
    } else {
        return basic+" if only {"+assertion.continuing.map(i=>candidate_names[i]).join(",")+"} remain";
    }
}



// Above this line are utilities.
// Below this line are GUI stuff


/**
 * Draw all elimination orders into the given <div/>
 * @param {Element} div The DOM element where things should be inserted
 * @param {number[][]} elimination_orders A list of still valid elimination orders
 * @param {string[]} candidate_names : a list of the candidate names
 * @param {number} num_candidates The number of candidates
 * @param {{winner:number,loser:number,continuing:number[],type:string}} [assertion] The optional assertion used to color code and annotate paths.
 */
function draw_trees(div,elimination_orders,candidate_names,num_candidates,assertion) {
    for (const eo of elimination_orders) {
        const line = add(div,"div");
        if (eo.length<num_candidates) add(line,"span").innerText="...<";
        for (let i=0;i<eo.length;i++) {
            const candidate = eo[i];
            if (i!==0) add(line,"span").innerText="<";
            let annotation = "candidate_name";
            if (assertion) {
                if (candidate===assertion.winner) annotation+=" winner";
                else if (candidate===assertion.loser) annotation+=" loser";
                else if (assertion.continuing) {
                    if (assertion.continuing.includes(candidate)) annotation+=" continuing";
                    else annotation+=" eliminated";
                }
            }
            add(line,"span",annotation).innerText=candidate_names[candidate];
        }
    }
}

/**
 * Make a human readable explanation of what the assertions imply
 * @param {Element} div The DOM element where things should be inserted
 * @param {{winner:number,loser:number,continuing:number[],type:string}[]} assertions The list of assertions
 * @param {string[]} candidate_names : a list of the candidate names
 * @param {boolean} expand_fully_at_start If true, expand all num_candidates factorial paths. If false, use minimal elimination order suffixes (tree prefixes) where possible.
 */
function explain(div,assertions,candidate_names,expand_fully_at_start) {
    const num_candidates=candidate_names.length;
    console.log(candidate_names);
    console.log(assertions);
    removeAllChildElements(div);
    let elimination_orders = expand_fully_at_start?all_elimination_orders(num_candidates):all_elimination_order_suffixes(num_candidates);
    add(div,"h5","explanation_text").innerText="We start with all possible elimination orders";
    draw_trees(add(div,"div","all_trees"),elimination_orders,candidate_names,num_candidates);
    for (const assertion of assertions) {
        add(div,"h4","assertion_name").innerText="Assertion : "+assertion_description(assertion,candidate_names);
        elimination_orders = assertion_all_allowed_suffixes(assertion,elimination_orders,num_candidates,true);
        add(div,"h5","explanation_text").innerText="Before applying assertion";
        draw_trees(add(div,"div","all_trees"),elimination_orders,candidate_names,num_candidates,assertion);
        elimination_orders = assertion_all_allowed_suffixes(assertion,elimination_orders,num_candidates,false);
        add(div,"h5","explanation_text").innerText="After applying assertion";
        draw_trees(add(div,"div","all_trees"),elimination_orders,candidate_names,num_candidates);
    }
}
