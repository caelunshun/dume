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
-- handleEvent(event, cv) - handles an event.
--
-- Widget auto-members:
-- contains(pos: Vector) -> bool - returns whether the given position is inside the widget's rectangle bounds.
--
-- `init` is optional and defaults to a no-op
-- `paint` is optional and defaults to painting all children.
-- `layout` is optional and defaults to laying out all children at the
-- same position as their parent. (Best for single-child or zero-child widgets.)
-- `handleEvent` is optional and defaults to calling handleEvent on all children.
--
-- size - the computed layout size
-- pos - the computed position relative to the parent
-- children - a list of widget children
-- params - provided when the widget is built
-- state - persistent state
-- style - current style for painting
-- classes - list of strings containing style classes for the widget
-- pressed, hovered - booleans indicating these states. They determine widget styling.
--
-- Widgets should not add more fields; they should keep their state within the `state` table.
--
-- In painting and layout, all widgets operate in a coordinate space
-- where their position is the origin.
--
-- # Events
-- An event is a table with a `type` field. The type determines
-- the remaining fields and can have one of the following values:
-- * "key"
--    * key: Key
--    * action: Action
--    * modifers: Modifiers (table)
-- * "char"
--    * char: number (UTF32)
-- * "cursorMove"
--   * pos: Vector
-- * "mouseClick"
--   * pos: Vector
--   * mouse: Mouse
--   * action: Action
--   * modifiers: Modifiers (table)
--  * "scroll"
--    * offset: Vector


local dume = {}

-- An entire UI tree.
local UI = {}

local Vector = require("brinevector")

function math.clamp(x, min, max)
    if x < min then return min end
    if x > max then return max end
    return x
end

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

local EventType = {
    Key = "key",
    Char = "char",
    CursorMove = "cursorMove",
    MouseClick = "mouseClick",
    Scroll = "scroll",
}
dume.EventType = EventType

local Key = {
    -- These values intentionally match GLTF keycodes.
    Space = 32,
    Apostrophe = 39,
    Comma = 44,
    Minus = 45,
    Period = 46,
    Slash = 47,
    N0 = 48,
    N1 = 49,
    N2 = 50,
    N3 = 51,
    N4 = 52,
    N5 = 53,
    N6 = 54,
    N7 = 55,
    N8 = 56,
    N9 = 57,
    Semicolon = 59,
    Equal = 61,
    A = 65,
    B = 66,
    C = 67,
    D = 68,
    E = 69,
    F = 70,
    G = 71,
    H = 72,
    I = 73,
    J = 74,
    K = 75,
    L = 76,
    M = 77,
    N = 78,
    O = 79,
    P = 80,
    Q = 81,
    R = 82,
    S = 83,
    T = 84,
    U = 85,
    V = 86,
    W = 87,
    X = 88,
    Y = 89,
    Z = 90,
    LeftBracket = 91,
    Backslash = 92,
    RightBracket = 93,
    Backtick = 96, -- `
    Escape = 256,
    Enter = 257,
    Tab = 258,
    Backspace = 259,
    Insert = 260,
    Delete = 261,
    Right = 262,
    Left = 263,
    Down = 264,
    Up = 265,
    PageUp = 266,
    PageDown = 267,
    Home =  268,
    End = 269,
    CapsLock = 280,
    ScrollLock = 281,
    NumLock = 282,
    PrintScreen = 283,
    Pause = 284,
    F1 = 290,
    F2 = 291,
    F3 = 292,
    F4 = 293,
    F5 = 294,
    F6 = 295,
    F7 = 296,
    F8 = 297,
    F9 = 298,
    F10 = 299,
    F11 = 300,
    F12 = 301,
    LShift = 340,
    LControl = 341,
    LAlt = 342,
}
dume.Key = Key

local Action = {
    Release = 0,
    Press = 1,
    Repeat = 2,
}
dume.Action = Action

local Mouse = {
    Left = 0,
    Right = 1,
    Middle = 2,
}
dume.Mouse = Mouse

local FontWeight = {
    Thin = 0,
    ExtraLight = 1,
    Light = 2,
    Normal = 3,
    Medium = 4,
    SemiBold = 5,
    Bold = 6,
    ExtraBold = 7,
    Black = 8,
}
dume.FontWeight = FontWeight

local FontStyle = {
    Normal = 0,
    Italic = 1,
}
dume.FontStyle = FontStyle

local function cross(axis)
    if axis == Axis.Horizontal then return Axis.Vertical
    else return Axis.Horizontal
    end
end
dume.cross = cross

-- Creates a new UI.
--
-- `style` should be a table with members default, hovered, and pressed,
-- and each of those fields should contain a mapping from class name => applied style parameters.
function UI:new(cv, style)
    local o = { cv = cv, style = style, windows = {} }

    setmetatable(o, self)
    self.__index = self

    return o
end

function UI:createWindow(name, pos, size, rootWidget)
    self:inflate(rootWidget)

    self.windows[name] = {
        name = name,
        pos = pos,
        size = size,
        rootWidget = rootWidget,
    }
end

function UI:deleteWindow(name)
    self.windows[name] = nil
end

function UI:handleEvent(event)
    for _, window in pairs(self.windows) do
        if event.pos ~= nil then
            event.pos = event.pos - window.pos
        end
        window.rootWidget:handleEvent(event, self.cv)
        if event.pos ~= nil then
            event.pos = event.pos + window.pos
        end
    end
end

function UI:render()
    self:computeWidgetLayouts()
    self:paintWidgets()
end

function UI:computeWidgetLayouts()
    for _, window in pairs(self.windows) do
        window.rootWidget.pos = window.pos
        window.rootWidget:layout(window.size, self.cv)
    end
end

function UI:paintWidgets()
    for _, window in pairs(self.windows) do
        self.cv:translate(window.pos)
        window.rootWidget:paint(self.cv)
        self.cv:translate(-window.pos)
    end
end

function UI:inflate(widget, parent)
    widget.children = widget.children or {}
    widget.offsetFromParent = widget.offsetFromParent or Vector(0, 0)

    -- Set default methods
    widget.contains = function(self, pos)
        return dume.rectContains(Vector(0, 0), self.size, pos)
    end

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

    widget.layoutChildren = function(self, maxSize, cv)
        local biggestSize = Vector(0, 0)
        for _, child in ipairs(self.children) do
            child:layout(maxSize, cv)
            child.pos = Vector(0, 0)

            if child.size.x > biggestSize.x then biggestSize.x = child.size.x end
            if child.size.y > biggestSize.y then biggestSize.y = child.size.y end

            self.offsetFromParent = child.offsetFromParent
        end
        return biggestSize
    end

    widget.layout = widget.layout or function(self, maxSize, cv)
        local biggestSize = self:layoutChildren(self.style.minSize or maxSize, cv)

        if self.fillParent then
            -- grow to the maximum possible size
            self.size = maxSize
        elseif self.style.minSize ~= nil then
            -- shrink to child size but with a minimum size
            self.size = Vector(
                    math.max(self.style.minSize.x, biggestSize.x),
                    math.max(self.style.minSize.y, biggestSize.y)
            )
        else
            -- shrink to child size
            self.size = biggestSize
        end
    end

    widget.invokeChildrenEvents = function(self, event, cv)
        for _, child in ipairs(self.children or {}) do
            -- Transform event position to child space
            if event.pos ~= nil then event.pos = event.pos - child.pos end

            child:handleEvent(event, cv)

            if event.pos ~= nil then event.pos = event.pos + child.pos end
        end
    end

    widget.handleEvent = widget.handleEvent or widget.invokeChildrenEvents
    local handler = widget.handleEvent

    widget.handleEvent = function(self, event, cv)
        handler(self, event, cv)

        if event.type == dume.EventType.CursorMove then
            self.hovered = self:contains(event.pos)
        end
        if event.type == dume.EventType.MouseClick then
            if (event.action == dume.Action.Press or event.action == dume.Action.Repeat) and self:contains(event.pos) then
                self.pressed = true
            else
                self.pressed = false
            end
        end

        if self.pressed then
            self.style = self._dumePressedStyle
        elseif self.hovered then
            self.style = self._dumeHoveredStyle
        else
            self.style = self._dumeDefaultStyle
        end
    end

    -- Styling
    local defaultStyle = {}
    local hoveredStyle = {}
    local pressedStyle = {}
    for _, class in ipairs(widget.classes or {}) do
        for k, v in pairs(self.style.default[class] or {}) do
            defaultStyle[k] = v
        end
        for k, v in pairs(self.style.hovered[class] or {}) do
            hoveredStyle[k] = v
        end
        for k, v in pairs(self.style.pressed[class] or {}) do
            pressedStyle[k] = v
        end
    end

    -- Inherit styling
    setmetatable(hoveredStyle, hoveredStyle)
    hoveredStyle.__index = function(t, k)
        return rawget(t, k) or defaultStyle[k]
    end
    setmetatable(pressedStyle, pressedStyle)
    pressedStyle.__index = function(t, k)
        return rawget(t, k) or hoveredStyle[k]
    end
    widget._dumeDefaultStyle = defaultStyle
    widget._dumeHoveredStyle = hoveredStyle
    widget._dumePressedStyle = pressedStyle
    widget.style = defaultStyle -- changed by handleEvent

    -- Initialize widget and inflate children
    if widget.init ~= nil then
        widget:init(self.cv)
    end
    for _, child in ipairs(widget.children) do
        self:inflate(child, widget)
    end
end

function UI:resize(oldCanvasSize, newCanvasSize)
    for _, window in pairs(self.windows) do
        window.size.x = window.size.x * newCanvasSize.x / oldCanvasSize.x
        window.size.y = window.size.y * newCanvasSize.y / oldCanvasSize.y
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
        return
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

function Canvas:circle(center, radius)
    self:arc(center, radius, 0, 2 * math.pi)
end

function dume.rectContains(rectPos, rectSize, pos)
    local rectEnd = rectPos + rectSize
    return pos.x >= rectPos.x and pos.y >= rectPos.y
        and pos.x <= rectEnd.x and pos.y <= rectEnd.y
end

return dume
