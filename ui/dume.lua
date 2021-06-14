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
-- build() - called every frame to build children. Returns a list of children. (optional, default returns the empty list)
-- init(cv) - called to compute state from initial parameters. This is called only
-- if the widget was added to the tree or was recreated, instead of every frame.
-- recreate() - clears the state table, then calls init()
-- paint(cv) - paints the widget to a canvas. should paint children too
-- layout(maxSize) - lays out the widget's children by setting their `pos` and `size` fields. Should set `self.size`
-- to the size of the widget.
--
-- `paint` is optional and defaults to painting all children.
-- `layout` is optional and defaults to laying out all children at the
-- same position as their parent. (Best for single-child or zero-child widgets.)
--
-- size - the computed layout size
-- pos - the computed position relative to the parent
-- children - a list of widget children
-- params - provided when the widget is built
-- state - persistent state
-- style - style for painting
--
-- Widgets should not add more fields; they should keep their state within the `state` table.
--
-- In painting and layout, all widgets operate in a coordinate space
-- where their own origin is the origin.

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

local Axis = {
    Horizontal = "x",
    Vertical = "y",
}
dume.Axis = Axis

local function cross(axis)
    if axis == Axis.Horizontal then return Axis.Vertical
    else return Axis.Horizontal
    end
end
dume.cross = cross

function UI:new(cv)
    local o = { cv = cv, style = {} }

    setmetatable(o, self)
    self.__index = self

    return o
end

function UI:begin()
    self.widgets = {}
    self.windows = {}
end

function UI:window(pos, size, rootWidget)
    self:inflate(rootWidget)

    self.windows[#self.windows + 1] = {
        pos = pos,
        size = size,
        rootWidget = rootWidget,
    }
end

function UI:render()
    self:computeWidgetLayouts()
    self:paintWidgets()
end

function UI:computeWidgetLayouts()
    for _, window in ipairs(self.windows) do
        window.rootWidget.pos = window.pos
        window.rootWidget:layout(window.size)
    end
end

function UI:paintWidgets()
    for _, window in ipairs(self.windows) do
        window.rootWidget:paint(self.cv)
    end
end

function UI:inflate(widget, parent)
    self.widgets[#self.widgets + 1] = widget

    widget.paintChildren = function(self, cv)
        for _, child in ipairs(self.children) do
            cv:translate(child.pos)
            child:paint(cv)
            cv:translate(-child.pos)
        end
    end

    widget.paint = widget.paint or function(self, cv)
        self:paintChildren(cv)
    end

    widget.layout = widget.layout or function(self, maxSize)
        for _, child in ipairs(self.children) do
            child.layout(maxSize)
            child.pos = Vector(0, 0)
        end
        self.size = maxSize
    end

    widget.style = widget.style or {}

    local parentStyle = nil
    if parent ~= nil then
        parentStyle = parent.style
    else
        parentStyle = self.style
    end

    for k, v in ipairs(parentStyle) do
        if widget.style[k] == nil then
            widget.style[k] = v
        end
    end

    widget:init(self.cv)
    local children = widget:build()
    widget.children = children
    for _, child in ipairs(children) do
        self:inflate(child, widget)
    end
end

dume.UI = UI

function dume.rgb(r, g, b, a)
    a = a or 255
    return {
        r = r,
        g = g,
        b = b,
        a = a,
    }
end

return dume
