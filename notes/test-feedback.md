Carefully consider the feedback. Then mark as completed only when issue is fully resolved.

# High Priority


- [ ] Item A: From the graph view, user should be able to click on a node and then create a new node attached to it from a dropdown of nodes. Add this under the actions menu, and then also offer the actions menu as the dropdown after right clicking on a node.

- [ ] Item B: Bugfix: Item names in table view are not shown. Seems like i can see a little bit of text on the left with enough length but they are getting covered up. 

- [ ] Item C: Make a "zen" mode, toggleable from the command pallete, that acts as a visual centered dashboard for the whole realm. At the center it includes the graph view, except all nodes are more geometric (squares, diamonds, circles) and edges connect to node centers. Nodes and edges are also kind of glowy to get a high tech look. Include other necessary information around the screen but be minimalistic. Zen mode is read only. Watch out for unecessary code repetition.

# Medium Priority

- [x] Bug1: Custom header at the top in desktop view works but doesn't allow dragging for movement of the window.
- [x] Bug2: Minimize, maximize, close buttons on the desktop version don't work
- [x] Bug3: Fibonacci quest points should not be enforced, just recommended. Replace the quest points textbox with button options for fibonacci amounts: 1,2,3,5,8... Then on the last box is a textbox where the user can enter any custom amount between 0 and 9999

- [x] Bug4: Highlighting epics or disciplines under a node and then clicking save components will add those same items, duplicating them and causing some weird bugs. Don't allow duplicates, but this behavior doesn't make sense either. Instead there should be an X popup when hovering over epics and disciplines allowing user to remove them. And adding new ones should come from pressing a plus button to open a dropdown menu.

- [x] Bug5: UI for node items in graph view needs improvements:
     - [x] Remove the colored dot, no use
     - [x] Sometimes edges (the lines that connect nodes) don't connect correctly to the node corners
     - [x] For nodes that visually have little height, it looks weird when edges connect to corners. Instead all edges should connect to the middle part of nodes (left center, right center, center top, center bottom)

- [x] Bug6: Legend for graph view sometimes is in the way but its still important information. Allow the user to close and reopen it.
- [x] Bug7: Explorer, graph, and table view buttons go to the same page. Just get rid of graph and table view since explorer already has them all.
- [x] Bug8: Settings button should be under the File dropdown, not on the sidebar.

- [x] Bug9: After double clicking on a node for the first time, all later clicks on a node are counted as double clicks on the first click instead of the second.

- [ ] Feature1: User can zoom in and out of the graph using scroll wheel, but on laptop trackpad it seems to be activated by using two fingers going up and down, which is intuitive and hard to control. Instead it should be controllable by pinching in and out. 