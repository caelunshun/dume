-- A widget that displays a different child depending on a set string.
local Navigator = {}

local Vector = require("brinevector")

function Navigator:new(pages, defaultPage)
    local o = { params = { pages = pages }, children = {}, currentPage = pages[defaultPage] }
    for pageName, page in pairs(pages) do
        o.children[#o.children + 1] = page
        page._navigatorPageName = pageName
    end
    setmetatable(o, self)
    self.__index = self
    return o
end

function Navigator:layout(maxSize, cv)
    self.currentPage:layout(maxSize, cv)
    self.size = self.currentPage.size
    self.currentPage.pos = Vector(0, 0)
end

function Navigator:paint(cv)
    self.currentPage:paint(cv)
end

function Navigator:handleEvent(cv)
    self.currentPage:handleEvent(cv)
end

function Navigator:setPage(page)
    self.currentPage = self.params.pages[page]
    assert(self.currentPage ~= nil, "navigator: unknown page " .. page)
end

function Navigator:getPage()
    return self.currentPage._navigatorPageName
end

return Navigator
