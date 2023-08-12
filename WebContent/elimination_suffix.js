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
    const basic = candidate_names[assertion.winner]+" beats "+candidate_names[assertion.loser]+" always";
    if (assertion.type==="NEB") {
        return basic;
    } else {
        return basic+" if only {"+assertion.continuing.map(i=>candidate_names[i]).join(",")+"} remain";
    }
}

class EliminationTreeNode {

    /**
     * Make a new tree node representing a candidate
     * @param {number} body
     */
    constructor(body) {
        this.body=body;
        this.children={};
        this.valid=false;
    }

    /**
     * Add a partial elimination order, last being the highest eliminated after this body to the tree.
     * @param {number[]} partial_elimination_order
     */
    addPath(partial_elimination_order) {
        if (partial_elimination_order.length===0) return;
        const last = partial_elimination_order[partial_elimination_order.length-1];
        const remaining = partial_elimination_order.slice(0,partial_elimination_order.length-1);
        if (!this.children.hasOwnProperty(last)) this.children[last]=new EliminationTreeNode(last);
        this.children[last].addPath(remaining);
    }

    /**
     * Annotate an existing path to be "valid".
     * @param {number[]} partial_elimination_order
     */
    validPath(partial_elimination_order) {
        this.valid = true;
        if (partial_elimination_order.length===0) return;
        const last = partial_elimination_order[partial_elimination_order.length-1];
        const remaining = partial_elimination_order.slice(0,partial_elimination_order.length-1);
        if (!this.children.hasOwnProperty(last)) this.children[last]=new EliminationTreeNode(last);
        this.children[last].validPath(remaining);
    }

    /**
     * Get the children of this node
     * @returns {EliminationTreeNode[]}
     */
    get orderedChildren() {
        let res = Object.values(this.children);
        res.sort( (a,b) => a.body-b.body );
        return res;
    }

    /**
     * The maximum height of the tree, in number of nodes.
     * @returns {number}
     */
    get height() {
        let max_child_height = 0;
        for (const c of Object.values(this.children)) max_child_height=Math.max(max_child_height,c.height);
        return 1+max_child_height;
    }
}

/**
 * After this call, this tree and its children will occupy horizontal space from tree_node.start_x to tree_node.start_x+tree_node.width.
 * @param {{orderedChildren:{}[],width:number,start_x:number}} tree_node The node of a tree. It (and all its children) will be assigned start_x and width fields.
 * @param {number} start_x The number of nodes to the left of this node when drawn on a tree.
 * @returns {number} The width (in units of number of nodes) that this tree occupies
 */
function computeWidthsForTreeNode(tree_node,start_x) {
    tree_node.start_x = start_x;
    let width = 0;
    for (const c of tree_node.orderedChildren) {
        const cw = computeWidthsForTreeNode(c,start_x);
        start_x+=cw;
        width+=cw;
    }
    tree_node.width = Math.max(1,width);
    return tree_node.width;
}

/**
 * A second way of viewing things is a tree which shows what assertions stopped it.
 * Each tree either has children, or has an assertion showing what stopped it, or is valid.
 * A tree with a valid child will also be valid.
 */
class TreeShowingWhatEliminatedItNode {
    /**
     * Make a new tree showing all paths until they can be elimi
     * @param {number[]} parent_elimination_order_suffix A suffix of the elimination order corresponding to the parent of this. [] if no parent.
     * @param {number} body The candidate index this node represents
     * @param {{winner:number,loser:number,continuing:number[],type:string}[]} assertions A list of all the assertions that may apply to this tree
     * @param {number} num_candidates The total number of candidates.
     * */
    constructor(parent_elimination_order_suffix,body,assertions,num_candidates) {
        const elimination_order_suffix = [body].concat(parent_elimination_order_suffix);
        this.body = body;
        this.orderedChildren = [];
        const assertions_requiring_more_info = [];
        for (const assertion of assertions) {
            let effect = assertion_ok_elimination_order_suffix(assertion,elimination_order_suffix);
            if (effect===EffectOfAssertionOnEliminationOrderSuffix.Contradiction) {
                this.assertion = assertion;
                this.valid = false;
                return;
            } else if (effect===EffectOfAssertionOnEliminationOrderSuffix.Ok) {  } // ignore it
            else { // must need more information.
                assertions_requiring_more_info.push(assertion);
            }
        }
        // if we got here, nothing contradicted it.
        if (assertions_requiring_more_info.length===0) { // nothing required more info => everything OK
            this.valid=true;
        } else {
            // we need to get more info
            this.valid = false; // may be changed if any child is valid.
            for (let candidate=0;candidate<num_candidates;candidate++) {
                if (elimination_order_suffix.includes(candidate)) continue;
                let child = new TreeShowingWhatEliminatedItNode(elimination_order_suffix,candidate,assertions_requiring_more_info,num_candidates);
                if (child.valid) this.valid=true;
                this.orderedChildren.push(child);
            }
        }
    }

    /**
     * The maximum height of the tree, in number of nodes.
     * @returns {number}
     */
    get height() {
        let max_child_height = 0;
        for (const c of this.orderedChildren) max_child_height=Math.max(max_child_height,c.height);
        return 1+max_child_height;
    }
}



// utilities for drawing trees below here.
/**
 * Get a list of all the trees (one for each possible winning candidate)
 * @param {number[][]} elimination_orders A list of still valid elimination orders
 * @param {number[][]} [after_applying_assertion_elimination_orders] A list of still valid elimination orders after the next assertion.
 * @returns {EliminationTreeNode[]}
 */
function make_trees(elimination_orders,after_applying_assertion_elimination_orders) {
    let root = new EliminationTreeNode(null);
    for (const elimination_order of elimination_orders) root.addPath(elimination_order);
    let valid_orders = Array.isArray(after_applying_assertion_elimination_orders)?after_applying_assertion_elimination_orders:elimination_orders;
    for (const elimination_order of valid_orders) root.validPath(elimination_order);
    return root.orderedChildren;
}


// Above this line are utilities.
// Below this line are GUI stuff


/**
 * Draw all elimination orders into the given <div/> as text
 * @param {Element} div The DOM element where things should be inserted
 * @param {number[][]} elimination_orders A list of still valid elimination orders
 * @param {string[]} candidate_names : a list of the candidate names
 * @param {{winner:number,loser:number,continuing:number[],type:string}} [assertion] The optional assertion used to color code and annotate paths.
 */
function draw_trees_as_text(div,elimination_orders,candidate_names,assertion) {
    for (const eo of elimination_orders) {
        const line = add(div,"div");
        if (eo.length<candidate_names.length) add(line,"span").innerText="...<";
        for (let i=0;i<eo.length;i++) {
            const candidate = eo[i];
            if (i!==0) add(line,"span").innerText="<";
            let annotation = "candidate_name "+candidate_class(candidate,assertion);
            add(line,"span",annotation).innerText=candidate_names[candidate];
        }
    }
}

/**
 * Draw a single tree as an SVG node.
 * @param {Element} div The DOM element where things should be inserted
 * @param {EliminationTreeNode} tree
 * @param {string[]} candidate_names : a list of the candidate names
 * @param {{winner:number,loser:number,continuing:number[],type:string}} [assertion] The optional assertion used to color code and annotate paths.
 **/
function draw_svg_tree(div,tree,candidate_names,assertion) {
    //console.log(tree);
    computeWidthsForTreeNode(tree,0);
    let nodes_wide = tree.width;
    //let nodes_high = tree.height;
    //if (nodes_high<candidate_names.length) nodes_high+=1; // there will be implicit triangles below. Account for them.
    const pixels_per_node_x = 80;
    const pixels_per_node_y = 50;
    const node_radius = 5;
    let svg = addSVG(div,"svg");
    let names = addSVG(svg,"g");
    let lines = addSVG(svg,"g");
    let nodes = addSVG(svg,"g");
    svg.setAttribute("width",nodes_wide*pixels_per_node_x);
    // svg.setAttribute("height",nodes_high*pixels_per_node_y);
    function drawTree(node,nodes_above_me) {
        let cx = (node.start_x+node.width/2.0)*pixels_per_node_x;
        let cy = (nodes_above_me===0?0.6:(0.5+nodes_above_me))*pixels_per_node_y;
        let nodeC = addSVG(nodes,nodes_above_me===0?"rect":"circle",candidate_class(node.body,assertion));
        let name = addSVG(names,"text",candidate_class(node.body,assertion)+" "+(nodes_above_me===0?"above":"left"));
        name.textContent=candidate_names[node.body];
        if (nodes_above_me===0) { // draw square, name above
            nodeC.setAttribute("x",cx-node_radius);
            nodeC.setAttribute("y",cy-node_radius);
            nodeC.setAttribute("width",2*node_radius);
            nodeC.setAttribute("height",2*node_radius);
            name.setAttribute("x",cx);
            name.setAttribute("y",cy-2*node_radius);
        } else { // draw circle, name to left
            nodeC.setAttribute("cx",cx);
            nodeC.setAttribute("cy",cy);
            nodeC.setAttribute("r",node_radius);
            name.setAttribute("x",cx-2*node_radius);
            name.setAttribute("y",cy);
        }
        function drawLineTo(x,y,valid) { // draw a line from this element to a location
            let line = addSVG(lines,"line",valid?"valid":"invalid");
            line.setAttribute("x1",cx);
            line.setAttribute("y1",cy);
            line.setAttribute("x2",x);
            line.setAttribute("y2",y);
        }
        let max_child_y = 0;
        for (const c of node.orderedChildren) {
            let position = drawTree(c,nodes_above_me+1);
            drawLineTo(position.cx,position.cy,c.valid);
            if (position.max_y>max_child_y) max_child_y=position.max_y;
        }
        let end_y = cy+node_radius;
        if (nodes_above_me!==candidate_names.length-1 && node.orderedChildren.length===0) { // draw a triangle below.
            let top_y = cy+0.5*pixels_per_node_y;
            let triangle_height = 30;
            let triangle_half_width = 15;
            let bottom_y = top_y+triangle_height;
            drawLineTo(cx,top_y,node.valid);
            addSVG(nodes,"polygon",node.valid?"valid":"invalid").setAttribute("points",""+cx+","+top_y+" "+(cx-triangle_half_width)+","+bottom_y+" "+(cx+triangle_half_width)+","+bottom_y);
            const skipped_nodes = factorial(candidate_names.length-1-nodes_above_me);
            let count = addSVG(names,"text",node.valid?"valid":"invalid");
            count.textContent=""+skipped_nodes;
            count.setAttribute("x",cx);
            count.setAttribute("y",bottom_y-5);
            end_y = bottom_y;
        }
        if (node.assertion) { // explain why we stopped here.
            // compute the lines to show.
            const lines = [candidate_names[node.assertion.winner]+" > "+candidate_names[node.assertion.loser]];
            const continuing = node.assertion.continuing;
            if (continuing) {
                lines.push("continuing:")
                for (const candidate of continuing) lines.push(candidate_names[candidate]);
            }
            // show the lines
            end_y+=5;
            for (const line of lines) {
                let text = addSVG(names,"text","assertion "+node.assertion.type);
                text.textContent=line;
                end_y+=11;
                text.setAttribute("x",cx);
                text.setAttribute("y",end_y)
            }
        }
        return {cx:cx,cy:cy,max_y:Math.max(end_y,max_child_y)};
    }
    const max_y_used = drawTree(tree,0).max_y;
    svg.setAttribute("height",max_y_used+10);
}

/**
 * Compute n!
 * @param {number} n
 * @returns {number} n factorial
 */
function factorial(n) {
    if (n===0) return 1;
    else return n*factorial(n-1);
}


/**
 * Get the class description of the candidate
 * @param {number} candidate
 * @param {{winner:number,loser:number,continuing:number[],type:string}} [assertion] The optional assertion used to color code and annotate paths.
 * @returns {string}
 */
function candidate_class(candidate,assertion) {
    if (assertion) {
        if (candidate===assertion.winner) return "winner";
        else if (candidate===assertion.loser) return "loser";
        else if (assertion.continuing) {
            if (assertion.continuing.includes(candidate)) return "continuing";
            else return "eliminated";
        } else return "irrelevant"
    } else return "no_assertion";
}
/**
 * Draw all elimination orders into the given <div/>
 * @param {Element} div The DOM element where things should be inserted
 * @param {number[][]} elimination_orders A list of still valid elimination orders
 * @param {string[]} candidate_names : a list of the candidate names
 * @param {{winner:number,loser:number,continuing:number[],type:string}} [assertion] The optional assertion used to color code and annotate paths.
 * @param {number[][]} [elimination_orders] A list of still valid elimination orders after the assertion above is applied. Used for coloring
 */
function draw_trees_as_trees(div,elimination_orders,candidate_names,assertion,after_applying_assertion_elimination_orders) {
    let trees = make_trees(elimination_orders,after_applying_assertion_elimination_orders);
    for (const tree of trees) {
        draw_svg_tree(div,tree,candidate_names,assertion);
    }
}

/**
 * Make a human-readable explanation of what the assertions imply
 * @param {Element} div The DOM element where things should be inserted
 * @param {{winner:number,loser:number,continuing:number[],type:string}[]} assertions The list of assertions
 * @param {string[]} candidate_names : a list of the candidate names
 * @param {boolean} expand_fully_at_start If true, expand all num_candidates factorial paths. If false, use minimal elimination order suffixes (tree prefixes) where possible.
 * @param {boolean} draw_text_not_trees If true, draw as text (a list of all combinations) rather than as a SVG tree.
 * @param {boolean} hide_winner If true, don't bother drawing trees that imply the winner won. Technically this are unnecessary, but they can be useful for intuition and sanity checking
 * @param {number} winner_id 0 based integer saying who the winner is. Only used if hide_winner is true.
 */
function explain(div,assertions,candidate_names,expand_fully_at_start,draw_text_not_trees,hide_winner,winner_id) {
    const num_candidates=candidate_names.length;
    //console.log(candidate_names);
    //console.log(assertions);

    // Explain the elimination method.
    add(div,"h3").innerText="Demonstration method 1: Progressive Elimination"
    let draw_trees = draw_text_not_trees?draw_trees_as_text:draw_trees_as_trees;
    let elimination_orders = expand_fully_at_start?all_elimination_orders(num_candidates):all_elimination_order_suffixes(num_candidates);
    if (hide_winner) {
        elimination_orders=elimination_orders.filter(order=>order[order.length-1]!==winner_id);
    }
    add(div,"h5","explanation_text").innerText="We start with all possible elimination orders"+(hide_winner?" (except those compatible with "+candidate_names[winner_id]+" winning)":"");
    draw_trees(add(div,"div","all_trees"),elimination_orders,candidate_names);
    for (const assertion of assertions) {
        add(div,"h4","assertion_name").innerText="Assertion : "+assertion_description(assertion,candidate_names);
        elimination_orders = assertion_all_allowed_suffixes(assertion,elimination_orders,num_candidates,true);
        const elimination_orders_after = assertion_all_allowed_suffixes(assertion,elimination_orders,num_candidates,false);
        add(div,"h5","explanation_text").innerText="Evaluate assertion, expanding paths if necessary";
        draw_trees(add(div,"div","all_trees"),elimination_orders,candidate_names,assertion,elimination_orders_after);
        elimination_orders = elimination_orders_after;
        add(div,"h5","explanation_text").innerText="After applying assertion";
        draw_trees(add(div,"div","all_trees"),elimination_orders,candidate_names);
    }

    // Explain the elimination method.
    add(div,"h3").innerText="Demonstration method 2: Show what eliminated each possibility"
    for (let candidate=0;candidate<candidate_names.length;candidate++) {
        if (hide_winner && candidate===winner_id) continue;
        const tree = new TreeShowingWhatEliminatedItNode([],candidate,assertions,candidate_names.length);
        add(div,"h5","candidate_result").innerText=candidate_names[candidate]+(tree.valid?" is NOT ruled out by the assertions":" is ruled out by the assertions");
        draw_svg_tree(add(div,"div","all_trees"),tree,candidate_names,null);
    }

}


function describe_raire_result(output_div,explanation_div,data) {
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
        let heading_name = "Assertions";
        if (data.metadata.hasOwnProperty("contest")) heading_name+=" for "+data.metadata.contest;
        if (data.solution.Ok.hasOwnProperty("difficulty")) heading_name+=" - difficulty = "+data.solution.Ok.difficulty;
        add(output_div,"h3","Assertions").innerText=heading_name;
        for (const av of data.solution.Ok.assertions) {
            let adiv = add(output_div,"div");
            if (av.hasOwnProperty("difficulty")) add(adiv,"span","difficulty_start").innerText=""+av.difficulty; // Michelle's format doesn't have it for individual assertions
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
        if (!(Array.isArray(candidate_names) && candidate_names.length===data.solution.Ok.num_candidates)) {
            candidate_names = [];
            for (let i=0;i<parsed_input.num_candidates;i++) { candidate_names.push("Candidate "+i); }
        }
        if (data.metadata.hasOwnProperty("contest")) add(explanation_div,"h4").innerText="Contest : "+data.metadata.contest;
        const hide_winner = document.getElementById("HideWinner").checked;
        let winner_id = data.solution.Ok.winner;
        const assertions = data.solution.Ok.assertions.map(a=>a.assertion);
        /* code to deduce the winner if not present, but it should always be there.
        if (hide_winner && winner_id===undefined) {
            for (let candidate=0;candidate<candidate_names.length;candidate++) {
                const tree = new TreeShowingWhatEliminatedItNode([],candidate,assertions,candidate_names.length);
                if (tree.valid) {
                    if (winner_id===null) winner_id=candidate; else { add(explanation_div,"div","error").innerText="Could not determine winner from assertions" ; return; }
                }
            }
        }*/
        explain(explanation_div,assertions,candidate_names,document.getElementById("ExpandAtStart").checked,document.getElementById("DrawAsText").checked,hide_winner,winner_id);
    } else if (data.solution && data.solution.Err) {
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