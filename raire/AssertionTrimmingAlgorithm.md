# Assertion trimming algorithms

The RAIRE algorithm will generate a *sufficient* set of assertions to eliminate all possibilities
other than the claimed winner being the actual winner. Furthermore, the maximum *difficulty* of those
assertions will be a minimum. However, it will not always generate the *minimal* sufficient
set of assertions of minimal maximum difficulty. It frequently occurs that a possible elimination
order that is blocked off by one assertion will also end up being blocked by one or more other assertions that
were generated for the purpose of blocking off other possibilities. 

This redundancy is not a major problem, but it is undesirable. It makes the effort of verifying
correctness harder, and means there are more assertions to prove. The difficulty of the assertions
is generally the most important driver of auditing effort, but a smaller number of assertions will help.

For this reason we wish to remove the redundant assertions after they have been generated
by the main RAIRE algorithm.

We provide two algorithms for doing this, with different tradeoffs. They have a common 
theme of looking at the tree of elimination orders. 

Recall that the easiest manner to prove that a set of assertions will rule out all elimination orders other than
the claimed winner is to build a tree representing all elimination orders for candidates leading to outcomes
other than the claimed winner actually winning. Then cut off any branch at the point where an assertion
makes that elimination order impossible. If every possible elimination order in the tree is cut off, then the
claimed winner is indeed implied by the assertion. Of course, most assertions will cut off more than one elimination
order, often cutting off multiple large sections of the tree at once. 

Hereafter in this discussion this tree will just be referred to as "the tree".

Speed wise, the computer time taken to build this tree tends to be determined primarily by the size of
the tree. 

## Trim algorithm 1: Minimize Tree

The _minimize tree_ algorithm aims to minimize the size of this tree, even if that results in more assertions
being needed than the absolute minimal number. This is helpful for human visualization - and verification - of
the tree.

The basic process is to build up the tree from its root, stopping at a node as soon as an assertion rules out
the elimination orders defined by that node. We then conclude that this assertion is required. If more than one
assertion rules out the same node, then we conclude that at least one of these assertion rules is required. How
to choose which of them is described later.

Each node on this tree has been visited by the RAIRE algorithm already, so the total amount of work will
be no more, and generally less than that of RAIRE algorithm. So this trim algorithm is generally safe time-wise.

### How to choose which of the multiple assertions blocking a node to choose.

If more than one assertion rules out a node, then we need to choose which one to select. This can
make a difference to the number of assertions ending up chosen. For instance, suppose there were two
nodes, the first ruled out by either A1 or A2, and the second ruled out by A2. Clearly the minimal
set of assertions is just the set consisting of A2, but the answer is not obvious in more complex
examples. A simplistic rule of just choosing one arbitrarily whenever a node is encountered may have
chosen A1 at the first node. When the second node was encountered, A2 would also be needed, producing
a non-optimal solution.

In general, problems of this sort are very time-consuming to solve optimally, although tools exist that
can deal with many practical cases, and these were tried. It turned out, however, that a very simple 
and fast algorithm in practice ended up producing just as good results on every test case tried. So
this is used instead, and is probably pretty good if not perfect in practice.

We start of with the simple heuristic of dealing with any node that is only ruled out by one assertion. We
can clearly conclude that *that* assertion is needed. In a second pass, we go back to the other nodes
and consider them consecutively. Many may be satisfied already as we have already chosen one of the assertions
sufficient to rule it out. If not, we choose one assertion arbitrarily. 

In practice, this simple two pass approach seems to work well, possibly optimally.

## Trim algorithm 2: Minimize Assertions

The _minimize assertions_ algorithm aims to minimize the total number of assertions, even if that makes
the tree larger than it would be otherwise. This reduces the total amount of administration needed.

Again we build up the tree from the root, except this time we do not stop at a node as soon as an assertion
rules out the elimination orders defined by that node. Instead we continue while there are assertions relevant
to the tree. It may turn out that assertion A1 blocks a node, but that assertions A2 and A3 block all
the children of that node. In this case we say that either A1 is needed, or both A2 and A3 are needed. 
Of course the situation may be even more complex - it may be able to use both A4 and A5 instead of A3 as well.

Generating this tree can easily visit nodes that were not visited by the RAIRE algorithm. So it is possible
- and indeed moderately likely - that this algorithm will take longer than the assertion generation itself.

### How to choose which of the multiple assertions blocking a node to choose.

This is more complex than for the prior algorithm because instead of choices like A1 or A2 we have
choices like A1 or (A2 and (A3 or (A4 and A5))). However, the same simplistic two pass approach 
described above again always has produced an optimal result on all the test cases tried upon (a variety
of actual IRV elections).

When making an arbitrary decision, there is always at least one single assertion that will suffice;
one of these is chosen rather than a longer list.





