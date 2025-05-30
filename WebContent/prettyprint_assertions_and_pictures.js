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
        return basic+" always";
    } else {
        return basic+" if only {"+assertion.continuing.map(i=>candidate_names[i]).join(",")+"} remain";
    }
}

/**
 * Like assertion_description but add SVG pictures of triangles for winner and loser symbols.
 * @param {Element} where where to insert the text.
 * @param {{winner:number,loser:number,continuing:number[],type:string}} assertion The assertion.
 * @param {string[]} candidate_names : a list of the candidate names
 * @return {string} a text description of the assertion
 */
function assertion_description_with_triangles(where,assertion,candidate_names) {
    where.append(candidate_names[assertion.winner]);
    let svg1 = addSVG(where,"svg");
    svg1.setAttribute("width",20);
    svg1.setAttribute("height",20);
    drawWinnerOrLoserSymbol(svg1,10,10,"winner",10);
    where.append(" beats "+candidate_names[assertion.loser]);
    let svg2 = addSVG(where,"svg");
    svg2.setAttribute("width",20);
    svg2.setAttribute("height",20);
    drawWinnerOrLoserSymbol(svg2,10,10,"loser",10);
    if (assertion.type==="NEB") {
        where.append(" always");
    } else {
        where.append(" if only {"+assertion.continuing.map(i=>candidate_names[i]).join(",")+"} remain");
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
     * Make a new tree showing all paths until they can be eliminated
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
 * @param {{preventTextOverlapping:boolean,splitGreaterThanLines:boolean,showAssertionIndex:boolean,showAssertionText:boolean}} tree_ui_options UI options for drawing trees (not used in this function)
 */
function draw_trees_as_text(div,elimination_orders,candidate_names,assertion,after_applying_assertion_elimination_orders,description,tree_ui_options) {
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
 * Draw a triangle that is either pointing up or down.
 * @param {Element} where Where the triangle should be inserted
 * @param {number} cx x position of center
 * @param {number} cy y position of center
 * @param {"winner"|"loser"} candidateClass
 * @param {number} node_radius Roughly the radius of the triangle.
 */
function drawWinnerOrLoserSymbol(where,cx,cy,candidateClass,node_radius) {
    const direction = candidateClass==="winner"?-1:1;
    let top_y = cy+direction*node_radius;
    let triangle_half_width = node_radius;
    let bottom_y = cy-direction*node_radius;
    addSVG(where,"polygon",candidateClass).setAttribute("points",""+cx+","+top_y+" "+(cx-triangle_half_width)+","+bottom_y+" "+(cx+triangle_half_width)+","+bottom_y);
}

/**
 * Draw a single tree as an SVG node.
 * @param {Element} div The DOM element where things should be inserted
 * @param {EliminationTreeNode} tree
 * @param {string[]} candidate_names : a list of the candidate names
 * @param {string} image_name : a file name for downloading the image
 * @param {{preventTextOverlapping:boolean,splitGreaterThanLines:boolean,showAssertionIndex:boolean,showAssertionText:boolean}} tree_ui_options UI options for drawing the tree.
 * @param {{winner:number,loser:number,continuing:number[],type:string}} [assertion] The optional assertion used to color code and annotate paths.
 **/
function draw_svg_tree(div,tree,candidate_names,image_name,tree_ui_options,assertion) {
    //console.log(tree);
    computeWidthsForTreeNode(tree,0);
    let nodes_wide = tree.width;
    //let nodes_high = tree.height;
    //if (nodes_high<candidate_names.length) nodes_high+=1; // there will be implicit triangles below. Account for them.
    const pixels_per_node_x = 80;
    const pixels_per_node_y = 50;
    const node_radius = 5;
    let svg = addSVG(div,"svg");
    allImages.push({svg:svg,name:image_name});
    const lines = addSVG(svg,"g");
    const background_names = addSVG(svg,"g");
    const names = addSVG(svg,"g");
    const nodes = addSVG(svg,"g");
    /** Draw a tree
     * @param {{}} node The root of the tree being drawn
     * @param {number} nodes_above_me The number of nodes above this node. 0 for first node.
     * @param {start_x} The minimum x value to start drawing this tree at.
    // returns a structure containing
    // * cx : The x coordinate of the start of the tree
    // * cy : The y coordinate of the start of the tree
    // * max_y : The largest y coordinate of anything in this tree.
    // * max_x : The largest x coordinate of anything in this tree (including reasonable border).
        */
    function drawTree(node,nodes_above_me,start_x) {
        // layout algorithm : Draw children first to work out width.
        let max_child_y = 0;
        let max_x = start_x;
        let children_root_positions = [];
        for (const c of node.orderedChildren) {
            let position = drawTree(c,nodes_above_me+1,max_x);
            max_x=position.max_x;
            children_root_positions.push({cx:position.cx,cy:position.cy,valid:c.valid})
            if (position.max_y>max_child_y) max_child_y=position.max_y;
        }
        let work_when_cx_known = [];
        const provisional_cx = (node.start_x+node.width/2.0)*pixels_per_node_x;
        // We don't know where to position it horizontally until we know the label widths.
        // We need to place the labels at a position to know their width.
        // We need to know how to position it horizontally to place the labels.
        // Solution : make up a guess at cx, use it, and readjust if needed.
        function call_now_and_when_cx_known(f) {
            f(provisional_cx);
            work_when_cx_known.push(f);
        }
        function call_when_cx_known(f) {
            work_when_cx_known.push(f);
        }
        const cy = (nodes_above_me===0?0.6:(0.5+nodes_above_me))*pixels_per_node_y;
        const candidateClass = candidate_class(node.body,assertion);
        const isWinnerOrLoser = candidateClass==="winner" || candidateClass==="loser";
        let name = addSVG(names,"text",candidateClass+" "+(nodes_above_me===0?"above":"left"));
        name.textContent=candidate_names[node.body];
        if (nodes_above_me===0) { // draw (probably) square, name above
            call_now_and_when_cx_known(cx=>name.setAttribute("x",cx));
            name.setAttribute("y",cy-2*node_radius);
            if (tree_ui_options.preventTextOverlapping) max_x=Math.max(max_x,start_x+name.getBBox().width);
        } else { // draw (probably) circle, name to left
            call_now_and_when_cx_known(cx=>name.setAttribute("x",cx-2*node_radius));
            name.setAttribute("y",cy);
            if (tree_ui_options.preventTextOverlapping) max_x=Math.max(max_x,start_x+name.getBBox().width*2);
        }
        if (isWinnerOrLoser) { // draw triangle
            call_when_cx_known(cx=>drawWinnerOrLoserSymbol(nodes,cx,cy,candidateClass,node_radius*1.6));
        } else {
            let nodeC = addSVG(nodes,nodes_above_me===0?"rect":"circle",candidateClass);
            if (nodes_above_me===0) { // draw square, name above
                call_now_and_when_cx_known(cx=>nodeC.setAttribute("x",cx-node_radius));
                nodeC.setAttribute("y",cy-node_radius);
                nodeC.setAttribute("width",2*node_radius);
                nodeC.setAttribute("height",2*node_radius);
            } else { // draw circle, name to left
                call_now_and_when_cx_known(cx=>nodeC.setAttribute("cx",cx));
                nodeC.setAttribute("cy",cy);
                nodeC.setAttribute("r",node_radius);
            }
        }
        // draw lines to the children.
        function drawLineTo(x,y,valid,cx) { // draw a line from this element to a location
            let line = addSVG(lines,"line",valid?"valid":"invalid");
            line.setAttribute("x1",cx);
            line.setAttribute("y1",cy);
            line.setAttribute("x2",x);
            line.setAttribute("y2",y);
        }
        for (const c of children_root_positions) {
            call_when_cx_known(cx=>drawLineTo(c.cx,c.cy,c.valid,cx));
        }
        let end_y = cy+node_radius;
        if (nodes_above_me!==candidate_names.length-1 && node.orderedChildren.length===0) { // draw a triangle below.
            const top_y = cy+0.5*pixels_per_node_y;
            const triangle_height = 30;
            const triangle_half_width = 15;
            const bottom_y = top_y+triangle_height;
            call_when_cx_known(cx=>drawLineTo(cx,top_y,node.valid,cx));
            const polygon = addSVG(nodes,"polygon",node.valid?"valid":"invalid");
            call_now_and_when_cx_known(cx=>polygon.setAttribute("points",""+cx+","+top_y+" "+(cx-triangle_half_width)+","+bottom_y+" "+(cx+triangle_half_width)+","+bottom_y));
            const skipped_nodes = factorial(candidate_names.length-1-nodes_above_me);
            const count = addSVG(names,"text",node.valid?"valid":"invalid");
            count.textContent=""+skipped_nodes;
            call_now_and_when_cx_known(cx=>count.setAttribute("x",cx));
            count.setAttribute("y",bottom_y-5);
            end_y = bottom_y;
        }
        if (node.assertion && tree_ui_options.showAssertionIndex) { // explain why we stopped here.
            end_y+=19;
            let text = addSVG(names,"text","assertion_index");
            text.textContent=""+(1+node.assertion.assertion_index);
            call_now_and_when_cx_known(cx=>text.setAttribute("x",cx));
            text.setAttribute("y",end_y);
            // SVG CSS doesn't handle background color and borders for text objects. Make an explicit rect.
            call_when_cx_known(cx => {
                const border = 3;
                const box = text.getBBox();
                const rect = addSVG(background_names,"rect","assertion_index");
                rect.setAttribute("x", box.x-border);
                rect.setAttribute("y", box.y-border);
                rect.setAttribute("width", box.width+2*border);
                rect.setAttribute("height", box.height+2*border);
            });
            end_y+=2;
        }
        if (node.assertion && tree_ui_options.showAssertionText) { // explain why we stopped here.
            // compute the lines to show.
            const l1 = candidate_names[node.assertion.winner];
            const l2 = "> "+candidate_names[node.assertion.loser];
            const lines = tree_ui_options.splitGreaterThanLines?[l1,l2]:[l1+" "+l2];
            const continuing = node.assertion.continuing;
            if (continuing) {
                lines.push("continuing:")
                for (const candidate of continuing) lines.push(candidate_names[candidate]);
            }
            // show the lines
            end_y+=5;
            for (const line of lines) {
                const text = addSVG(names,"text","assertion "+node.assertion.type);
                text.textContent=line;
                end_y+=11;
                call_now_and_when_cx_known(cx=>text.setAttribute("x",cx));
                text.setAttribute("y",end_y);
                if (tree_ui_options.preventTextOverlapping) max_x=Math.max(max_x,start_x+text.getBBox().width);
            }
        }
        max_x = Math.max(max_x,start_x+pixels_per_node_x);
        const cx = (start_x+max_x)/2;
        for (const f of work_when_cx_known) f(cx);
        return {cx:cx,cy:cy,max_y:Math.max(end_y,max_child_y),max_x:max_x};
    }
    const space_used = drawTree(tree,0,0);
    svg.setAttribute("height",space_used.max_y+10);
    svg.setAttribute("width",space_used.max_x);
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
 * @param {number[][]} [after_applying_assertion_elimination_orders] A list of still valid elimination orders after the assertion above is applied. Used for coloring
 * @param description A text description of why these trees are being shown suitable for a file name
 * @param {{preventTextOverlapping:boolean,splitGreaterThanLines:boolean,showAssertionIndex:boolean,showAssertionText:boolean}} tree_ui_options UI options for drawing the tree.
 */
function draw_trees_as_trees(div,elimination_orders,candidate_names,assertion,after_applying_assertion_elimination_orders,description,tree_ui_options) {
    let trees = make_trees(elimination_orders,after_applying_assertion_elimination_orders);
    for (const tree of trees) {
        draw_svg_tree(div,tree,candidate_names,"Possible methods for "+candidate_names[tree.body]+" to win "+description,tree_ui_options,assertion);
    }
}

/**
 * Make a human-readable explanation of what the assertions imply
 * @param {Element} div The DOM element where things should be inserted
 * @param {{winner:number,loser:number,continuing:number[],type:string}[]} assertions The list of assertions. We will add an "assertion_index" field to each if it is not already there.
 * @param {string[]} candidate_names : a list of the candidate names
 * @param {boolean} expand_fully_at_start If true, expand all num_candidates factorial paths. If false, use minimal elimination order suffixes (tree prefixes) where possible.
 * @param {boolean} draw_text_not_trees If true, draw as text (a list of all combinations) rather than as a SVG tree.
 * @param {boolean} hide_winner If true, don't bother drawing trees that imply the winner won. Technically this are unnecessary, but they can be useful for intuition and sanity checking
 * @param {number} winner_id 0 based integer saying who the winner is. Only used if hide_winner is true.
 */
function explain(div,assertions,candidate_names,expand_fully_at_start,draw_text_not_trees,hide_winner,winner_id) {
    for (let assertion_index=0;assertion_index<assertions.length;assertion_index++) {
        const assertion = assertions[assertion_index];
        if (!assertion.hasOwnProperty("assertion_index")) assertion.assertion_index = assertion_index;
    }
    const num_candidates=candidate_names.length;
    //console.log(candidate_names);
    //console.log(assertions);
    function checkBoxIfPresent(boxName,default_value) {
        let ui = document.getElementById(boxName);
        return ui?ui.checked:default_value;
    }
    const show_separately = checkBoxIfPresent("ShowEffectOfEachAssertionSeparately",false);
    const tree_ui_options = {
        preventTextOverlapping : checkBoxIfPresent("preventTextOverlapping",true),
        splitGreaterThanLines : checkBoxIfPresent("splitGreaterThanLines",true),
        showAssertionIndex : checkBoxIfPresent("showAssertionIndex",true),
        showAssertionText : checkBoxIfPresent("showAssertionText",true),
    };
    allImages=[];
    if (show_separately) {
        // Explain the elimination method.
        add(div,"h3").innerText="Demonstration by progressive elimination"
        let draw_trees = draw_text_not_trees?draw_trees_as_text:draw_trees_as_trees;
        let elimination_orders = expand_fully_at_start?all_elimination_orders(num_candidates):all_elimination_order_suffixes(num_candidates);
        if (hide_winner) {
            elimination_orders=elimination_orders.filter(order=>order[order.length-1]!==winner_id);
        }
        add(div,"h5","explanation_text").innerText="We start with all possible elimination orders"+(hide_winner?" (except those compatible with "+candidate_names[winner_id]+" winning)":"");
        draw_trees(add(div,"div","all_trees"),elimination_orders,candidate_names,null,null,"at start",tree_ui_options);
        for (const assertion of assertions) {
            const assertionHeading = add(div,"h4","assertion_name");
            assertionHeading.append("Assertion : ");
            assertion_description_with_triangles(assertionHeading,assertion,candidate_names);
            elimination_orders = assertion_all_allowed_suffixes(assertion,elimination_orders,num_candidates,true);
            const elimination_orders_after = assertion_all_allowed_suffixes(assertion,elimination_orders,num_candidates,false);
            add(div,"h5","explanation_text").innerText="Evaluate assertion, expanding paths if necessary";
            draw_trees(add(div,"div","all_trees"),elimination_orders,candidate_names,assertion,elimination_orders_after,"before applying "+assertion_description(assertion,candidate_names),tree_ui_options);
            elimination_orders = elimination_orders_after;
            add(div,"h5","explanation_text").innerText="After applying assertion";
            draw_trees(add(div,"div","all_trees"),elimination_orders,candidate_names,null,null,"after applying "+assertion_description(assertion,candidate_names),tree_ui_options);
        }
    } else {
        // Explain the elimination method.
        add(div,"h3").innerText="Demonstration by showing what eliminated each possibility"
        for (let candidate=0;candidate<candidate_names.length;candidate++) {
            if (hide_winner && candidate===winner_id) continue;
            const tree = new TreeShowingWhatEliminatedItNode([],candidate,assertions,candidate_names.length);
            add(div,"h5","candidate_result").innerText=candidate_names[candidate]+(tree.valid?" is NOT ruled out by the assertions":" is ruled out by the assertions");
            draw_svg_tree(add(div,"div","all_trees"),tree,candidate_names,"Elimination tree for "+candidate_names[candidate],tree_ui_options,null);
        }
    }
    let save_images_button = add(div,"button");
    save_images_button.textContent="Save all images";
    save_images_button.addEventListener('click', saveAllImages);
}

function checkOptionVisibility() {
    const show_separately = document.getElementById("ShowEffectOfEachAssertionSeparately").checked;
    const applies_to = document.getElementById("IfShowEffectOfEachAssertionSeparately");
    const applies_to_inverse = document.getElementById("IfNotShowEffectOfEachAssertionSeparately");
    if (applies_to) applies_to.style.display=show_separately?"":"none";
    if (applies_to_inverse) applies_to_inverse.style.display=show_separately?"none":"";
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
        if (data.solution.Ok.warning_trim_timed_out) {
            add(output_div,"p","warning").innerText="Warning : Trimming timed out. Some assertions may be redundant.";
        }
        function describe_time(what,time_taken) {
            if (time_taken) {
                let time_desc = time_taken.seconds>0.1?Number(time_taken.seconds).toFixed(1)+" seconds":Number(time_taken.seconds*1000).toFixed(2)+" milliseconds";
                add(output_div,"p").innerText="Time to "+what+" : "+time_desc+" ("+time_taken.work+" operations)";
            }
        }
        describe_time("determine winners",data.solution.Ok.time_to_determine_winners);
        describe_time("find assertions",data.solution.Ok.time_to_find_assertions);
        describe_time("trim assertions",data.solution.Ok.time_to_trim_assertions);
        let heading_name = "Assertions";
        if (data.metadata.hasOwnProperty("contest")) heading_name+=" for "+data.metadata.contest;
        if (data.solution.Ok.hasOwnProperty("difficulty")) heading_name+=" - difficulty = "+data.solution.Ok.difficulty;
        if (data.solution.Ok.hasOwnProperty("margin")) heading_name+=" margin = "+data.solution.Ok.margin;
        add(output_div,"h3","Assertions").innerText=heading_name;
        let assertionRisks = data.metadata && data.metadata.assertionRisks; // a tool may add the risk limits from the audit to the metadata either here or in the status field for each assertion.
        let riskLimit = data.metadata && data.metadata.riskLimit;
        let assertionIndex = 0;
        let a_heading_div = add(output_div,"div");
        a_heading_div.style.fontWeight="bold";
        for (const av of data.solution.Ok.assertions) {
            let adiv = add(output_div,"div");
            // do index
            add(add(adiv,"span","ref_start"),"span","assertion_index").innerText=""+(assertionIndex+1);
            if (assertionIndex===0) {
                let ref_head = add(a_heading_div,"span","ref_start");
                ref_head.innerText="Ref";
                ref_head.title="The reference is a number that is used in the pictures below to refer to a specific assertion."
            }
            // do risk
            let risk = (av.hasOwnProperty("status") && av.status.hasOwnProperty("risk"))?av.status.risk:(
                (Array.isArray(assertionRisks) && assertionRisks.length>assertionIndex)?assertionRisks[assertionIndex]:null
            );
            if (typeof risk==="number") {
                let isGood = typeof riskLimit==="number"?(risk<=riskLimit?"risk_ok":"risk_bad"):"risk"
                let span = add(adiv,"span",isGood);
                span.innerText=""+risk;
                if (typeof riskLimit==="number") span.title="Risk limit = "+riskLimit;
                if (assertionIndex===0) {
                    add(adiv,"a_heading_div","risk").innerText="Difficulty";
                }
            }
            // do difficulty
            if (av.hasOwnProperty("difficulty")) { // Michelle's format doesn't have it for individual assertions
                add(adiv, "span", "difficulty_start").innerText = "" + av.difficulty;
                if (assertionIndex===0) add(a_heading_div, "span", "difficulty_start").innerText = "Difficulty";
            }
            // do margin
            if (av.hasOwnProperty("margin")) {  // Michelle's format (and old raire-rs) doesn't have it for individual assertions
                add(adiv, "span", "margin_start").innerText = "" + av.margin;
                if (assertionIndex===0) add(a_heading_div, "span", "margin_start").innerText = "Margin";
            }
            // do description
            const a = av.assertion;
            const adesc = add(adiv,"span");
            if (assertionIndex===0) add(a_heading_div, "span").innerText = "Description";
            if (a["type"] === "NEN") {
                adesc.innerText="NEN: "+candidate_name(a.winner)+" > "+candidate_name(a.loser)+" if only {"+candidate_name_list(a.continuing)+"} remain";
            } else if (a["type"] === "NEB") {
                adesc.innerText=candidate_name(a.winner)+" NEB "+candidate_name(a.loser);
            } else {
                adesc.innerText="Unknown assertion type"
            }
            assertionIndex++;
        }
        let candidate_names = data.metadata && data.metadata.candidates;
        if (!(Array.isArray(candidate_names))) candidate_names = [];
        for (let i=candidate_names.length;i<data.solution.Ok.num_candidates;i++) { candidate_names.push("Candidate "+i); } // extend names if only some present
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
        if (err==="InvalidCandidateNumber") {
            add(output_div, "p", "error").innerText = "Invalid candidate number in the preference list. Candidate numbers should be 0 to num_candidates-1 inclusive.";
        } else if (err==="InvalidNumberOfCandidates") {
            add(output_div, "p", "error").innerText = "Invalid number of candidates. There should be at least one candidate.";
        } else if (err==="TimeoutCheckingWinner") {
            add(output_div, "p", "error").innerText = "Timeout checking winner - either your problem is exceptionally difficult, or your timeout is exceedingly small.";
        } else if (err.hasOwnProperty("TimeoutFindingAssertions")) {
            add(output_div,"p","error").innerText="Timeout finding assertions - your problem is quite hard. Difficulty when interrupted : "+err.TimeoutFindingAssertions;
        } else if (err==="InvalidTimeout") {
            add(output_div,"p","error").innerText="Timeout is not valid. Timeout should be a number greater than zero.";
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