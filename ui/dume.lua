-- A UI library for games.
--
-- Each widget is a table with the following fields:
-- * `state`, which stores persistent widget state
-- * `params`, which stores user-provided parameters for the widget.
-- Initial state is a function of `params`.
-- * `style`, which contains style parameters for drawing. Style
-- is inherited from the parent.
--
-- Widget lifecycle:
-- * user invokes constructor and adds the widget to a window or a parent widget
-- * Dume invokes init()
-- * Each frame:
--    * layout()
--    * paint()
--  * widget destroyed - either because its parent died or by explicit destruction
--
-- Widget members:
-- init(cv) - called to compute state from initial parameters.
-- paint(cv) - paints the widget to a canvas. should paint children too
-- layout(maxSize, cv) - lays out the widget's children by setting their `pos` and `size` fields. Should set `self.size`
-- to the size of the widget.
--
-- `init` is optional and defaults to a no-op
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
    Middle = 1,
    Alphabetic = 2,
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
    local o = { cv = cv, style = {}, windows = {} }

    setmetatable(o, self)
    self.__index = self

    return o
end

function UI:createWindow(name, pos, size, rootWidget)
    self:inflate(rootWidget)

    table.insert(self.windows, {
        name = name,
        pos = pos,
        size = size,
        rootWidget = rootWidget,
    })
end

function UI:deleteWindow(name)
    self.windows[name] = nil
end

function UI:render()
    self:computeWidgetLayouts()
    self:paintWidgets()
end

function UI:computeWidgetLayouts()
    for _, window in ipairs(self.windows) do
        window.rootWidget.pos = window.pos
        window.rootWidget:layout(window.size, self.cv)
    end
end

function UI:paintWidgets()
    for _, window in ipairs(self.windows) do
        window.rootWidget:paint(self.cv)
    end
end

function UI:inflate(widget, parent)
    widget.children = widget.children or {}

    -- Set default methods
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

    widget.layout = widget.layout or function(self, maxSize, cv)
        for _, child in ipairs(self.children) do
            child:layout(maxSize, cv)
            child.pos = Vector(0, 0)
        end
        self.size = maxSize
    end

    -- Style inheritance
    widget.style = widget.style or {}

    local parentStyle = nil
    if parent ~= nil then
        parentStyle = parent.style
    else
        parentStyle = self.style
    end

    for k, v in pairs(parentStyle) do
        if widget.style[k] == nil then
            widget.style[k] = v
        end
    end

    -- Initialize widget and inflate children
    if widget.init ~= nil then
        widget:init(self.cv)
    end
    for _, child in ipairs(widget.children) do
        self:inflate(child, widget)
    end
end

dume.UI = UI

function dume.rgb(r, g, b, a)
    a = a or 255
    return { r, g, b, a }
end

-- Canvas extension methods for drawing high-level primitives.

function Canvas:rect(pos, size)
    self:moveTo(pos)
    self:lineTo(pos + Vector(size.x, 0))
    self:lineTo(pos + size)
    self:lineTo(pos + Vector(0, size.y))
    self:lineTo(pos)
end

function Canvas:roundedRect(pos, size, radius)
    if radius < 0.1 then
        self:rect(pos, size)
    end

    local offsetX = Vector(radius, 0)
    local offsetY = Vector(0, radius)

    local sizeX = Vector(size.x, 0)
    local sizeY = Vector(0, size.y)

    self:moveTo(pos + offsetX)
    self:lineTo(pos + sizeX - offsetX)
    self:quadTo(pos + sizeX, pos + sizeX + offsetY)
    self:lineTo(pos + size - offsetY)
    self:quadTo(pos + size, pos + size - offsetX)
    self:lineTo(pos + sizeY + offsetX)
    self:quadTo(pos + sizeY, pos + sizeY - offsetY)
    self:lineTo(pos + offsetY)
    self:quadTo(pos, pos + offsetX)
end

return dume
