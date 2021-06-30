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
    -- These values intentionally match Winit keycodes.
    Key1 = 0,
    Key2 = 1,
    Key3 = 2,
    Key4 = 3,
    Key5 = 4,
    Key6 = 5,
    Key7 = 6,
    Key8 = 7,
    Key9 = 8,
    Key0 = 9,
    A = 10,
    B = 11,
    C = 12,
    D = 13,
    E = 14,
    F = 15,
    G = 16,
    H = 17,
    I = 18,
    J = 19,
    K = 20,
    L = 21,
    M = 22,
    N = 23,
    O = 24,
    P = 25,
    Q = 26,
    R = 27,
    S = 28,
    T = 29,
    U = 30,
    V = 31,
    W = 32,
    X = 33,
    Y = 34,
    Z = 35,
    Escape = 36,
    F1 = 37,
    F2 = 38,
    F3 = 39,
    F4 = 40,
    F5 = 41,
    F6 = 42,
    F7 = 43,
    F8 = 44,
    F9 = 45,
    F10 = 46,
    F11 = 47,
    F12 = 48,
    F13 = 49,
    F14 = 50,
    F15 = 51,
    F16 = 52,
    F17 = 53,
    F18 = 54,
    F19 = 55,
    F20 = 56,
    F21 = 57,
    F22 = 58,
    F23 = 59,
    F24 = 60,
    Snapshot = 61,
    Scroll = 62,
    Pause = 63,
    Insert = 64,
    Home = 65,
    Delete = 66,
    End = 67,
    PageDown = 68,
    PageUp = 69,
    Left = 70,
    Up = 71,
    Right = 72,
    Down = 73,
    Back = 74,
    Return = 75,
    Space = 76,
    Compose = 77,
    Caret = 78,
    Numlock = 79,
    Numpad0 = 80,
    Numpad1 = 81,
    Numpad2 = 82,
    Numpad3 = 83,
    Numpad4 = 84,
    Numpad5 = 85,
    Numpad6 = 86,
    Numpad7 = 87,
    Numpad8 = 88,
    Numpad9 = 89,
    NumpadAdd = 90,
    NumpadDivide = 91,
    NumpadDecimal = 92,
    NumpadComma = 93,
    NumpadEnter = 94,
    NumpadEquals = 95,
    NumpadMultiply = 96,
    NumpadSubtract = 97,
    AbntC1 = 98,
    AbntC2 = 99,
    Apostrophe = 100,
    Apps = 101,
    Asterisk = 102,
    At = 103,
    Ax = 104,
    Backslash = 105,
    Calculator = 106,
    Capital = 107,
    Colon = 108,
    Comma = 109,
    Convert = 110,
    Equals = 111,
    Grave = 112,
    Kana = 113,
    Kanji = 114,
    LAlt = 115,
    LBracket = 116,
    LControl = 117,
    LShift = 118,
    LWin = 119,
    Mail = 120,
    MediaSelect = 121,
    MediaStop = 122,
    Minus = 123,
    Mute = 124,
    MyComputer = 125,
    NavigateForward = 126,
    NavigateBackward = 127,
    NextTrack = 128,
    NoConvert = 129,
    OEM102 = 130,
    Period = 131,
    PlayPause = 132,
    Plus = 133,
    Power = 134,
    PrevTrack = 135,
    RAlt = 136,
    RBracket = 137,
    RControl = 138,
    RShift = 139,
    RWin = 140,
    Semicolon = 141,
    Slash = 142,
    Sleep = 143,
    Stop = 144,
    Sysrq = 145,
    Tab = 146,
    Underline = 147,
    Unlabeled = 148,
    VolumeDown = 149,
    VolumeUp = 150,
    Wake = 151,
    WebBack = 152,
    WebFavorites = 153,
    WebForward = 154,
    WebHome = 155,
    WebRefresh = 156,
    WebSearch = 157,
    WebStop = 158,
    Yen = 159,
    Copy = 160,
    Paste = 161,
    Cut = 162,
}
dume.Key = Key

local Action = {
    Press = 0,
    Release = 1,
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

    self:computeWidgetLayouts()
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

    if event.pos ~= nil then
        for _, window in pairs(self.windows) do
            if dume.rectContains(window.pos, window.size, event.pos) then
                return true -- event may have affected this window
            end
        end
    end
    return false -- didn't handle event
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
