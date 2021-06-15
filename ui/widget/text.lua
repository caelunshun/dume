-- A widget that renders some rich text.
-- Note that text is stored in a markup format
-- that is subject to injection attacks. When including user-provided strings, use the
-- variable system (%<var> is replaced with the value of <var>)
-- to ensure security.
local Text = {}

local dume = require("dume")
local Vector = require("brinevector")

function Text:new(markup, variables, layout)
    local variables = variables or {}
    local layout = layout or {}

    layout.alignH = layout.alignH or dume.Align.Start
    layout.alignV = layout.alignV or dume.Align.Start
    if layout.lineBreaks == nil then layout.lineBreaks = true end
    layout.baseline = layout.baseline or dume.Baseline.Top

    layout.maxDimensions = Vector(math.huge, math.huge)

    local o = {
        params = {
            markup = markup,
            variables = variables,
            layout = layout
        },
        state = {}
    }
    setmetatable(o, self)
    self.__index = self
    return o
end

function Text:init(cv)
    self.state.text = cv:parseTextMarkup(self.params.markup, self.params.variables)
    self.state.paragraph = cv:createParagraph(self.state.text, self.params.layout)
end

function Text:layout(maxSize, cv)
    cv:resizeParagraph(self.state.paragraph, maxSize)
    self.size = Vector(cv:getParagraphWidth(self.state.paragraph), cv:getParagraphHeight(self.state.paragraph))
end

function Text:paint(cv)
    cv:drawParagraph(self.state.paragraph, Vector(0,0))
end

return Text
