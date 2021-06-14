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
        }
    }
    setmetatable(o, self)
    self.__index = self
    return o
end

function Text:init(cv)
    self.state.text = cv:createTextFromMarkup(self.params.markup, self.params.variables)
    self.state.paragraph = cv:createParagraph(self.state.text, self.params.layout)
end

function Text:layout(maxSize)
    self.state.paragraph:updateMaxSize(maxSize)
    self.size = Vector(self.state.paragraph:width(), self.state.paragraph:height())
end

function Text:paint(cv)
    cv:drawParagraph(self.pos, self.state.paragraph)
end

return Text
