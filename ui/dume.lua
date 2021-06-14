-- A UI library for games.
--
-- Dume exposes an immediate-mode API, but internally
-- the widget tree is retained mode.
--
-- Each widget is a table with the following fields:
-- * `state`, which stores persistent widget state
-- * `params`, which stores user-provided parameters for the widget.
-- Initial state is a function of `params`.
-- * `style`, which contains style parameters for drawing. Style
-- is inherited from the parent.
--
-- Each frame, the user code builds a widget tree by
-- creating widgets with initial parameters. Dume
-- then diffs the new parameters with the current ones
-- and calls the updated() method on a widget for each changed parameter.
-- If this function does not exist, the default behavior is to recreate the widget.
--
-- Widgets can create their own children in their build() method, which is called
-- each frame. They paint in the paint() method and handle input in the event() method.
--
-- Widget members:
-- updated(param_name, param_val) - update a parameter (optional, default behavior recreates)
-- build(ui) - called every frame to build children
-- init(cv) - called to compute state from initial parameters. This is called only
-- if the widget was added to the tree, instead of every frame.
-- recreate() - clears the state table, then calls init()
-- paint(cv) - paints the widget to a canvas. should paint children too
-- layout(max_size) - lays out the widget's children by setting their `bounds` fields. Should set `self.bounds.size`
-- to the size of the widget.
--
-- bounds - the computed layout bounds (size and pos)
-- children - a list of widget children
-- params - provided when the widget is built
-- state - persistent state
-- style - style for painting
--
-- Widgets should not add more fields; they should keep their state within the `state` table.

local dume = {}

-- An entire UI tree.
local UI = {}

local Vector = require("brinevector")

local Align = {
    Start = 0,
    Center = 1,
    End = 2,
}
dume.Align = Align

local Baseline = {
    Top = 0,
    Alphabetic = 1,
    Middle = 2,
    Bottom = 3,
}
dume.Baseline = Baseline

function UI:new()
    local o = {
        widgets = {}
    }

    setmetatable(o, self)
    self.__index = self

    return o
end

dume.UI = UI

return dume
