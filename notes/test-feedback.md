Carefully consider the feedback. Then mark as completed only when issue is fully resolved.

# High Priority

- [ ] A: Missing a lot of fundamental UX
    - [x] Bug A-1: No way to intuitively move between different root nodes in the graph view. For example, in the example realm, Character Movement is under Locomotion System under Combat Engine. However, there is no way to view switch between nodes under Locomation System nor Combat Engine. 
        - [x] Feature A-a: User should be able to go to a global graph view that shows every node in the entire realm.
        - [x] Feature A-b: Category organization
            - [x] Feature: Add a genre node that functions the same as a card node. But it will be used for broader categorizations like "Combat Engine" and "Locomotion System".
            - [x] Feature: User should be able to go to a kind of overview graph view that shows the root node (realm root) that branches off covering the entire realm, terminating after it hits a card node, action node, idea node, etc... The only node it doesn't terminate is when it hits a genre node, so the final result will show all genre nodes and their immediate children.
    - [ ] Bug A-2: User has no way to intuitively manipulate items. They should be able to create, delete, modify all types of nodes from all views (graph, table, board, matrix, etc...)
        - [ ] Bug: Plus button in board view does nothing
        - [ ] Bug: No edit button popup when hovering over MD description of a node
    - [ ] Bug A-3: When the user closes the filters menu in the explorer view, there is no way to open it up again
    - [ ] Bug A-4: Explorer view that merges multiple views is left mostly unimplemented. It opens graph and table by default, but there is no way to open new menus once one gets closed. Explorer view also doesnt save state when the user go to another view and them come back.

- [ ] Bug B: Item names in table view are not shown

# Medium Priority

- [ ] Bug: Custom header at the top in desktop view works but doesn't allow dragging for movement of the window.
- [ ] Bug: Minimize, maximize, close buttons on the desktop version don't work

