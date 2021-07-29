-- A tabular representation of data.
local Table = {}

local Vector = require("brinevector")
local Padding = require("widget/padding")
local Empty = require("widget/empty")

-- `columns` is an array of column names.
-- `rows` is an array of tables of (column name) -> (row widget).
function Table:new(columns, rows, cellPadding, numEmptyRows, minRowHeight)
    cellPadding = cellPadding or 5
    numEmptyRows = numEmptyRows or 1
    minRowHeight = minRowHeight or 30

    local theEmptyRow = {}
    for _, columnName in ipairs(columns) do
        theEmptyRow[columnName] = Empty:new()
    end

    for i=1,numEmptyRows do
        rows[#rows + 1] = theEmptyRow
    end

    local o = {
        fillParent = true,
        columns = columns,
        rows = rows,
        children = {},
        cellPadding = cellPadding,
        classes = { "table" },
        minRowHeight = minRowHeight,
    }
    for _, row in ipairs(rows) do
        for columnName, widget in pairs(row) do
            widget = Padding:new(widget, cellPadding)
            row[columnName] = widget
            o.children[#o.children + 1] = widget
        end
    end

    setmetatable(o, self)
    self.__index = self
    return o
end

function Table:layout(maxSize, cv)
    local cellSizes = {}
    for i, row in ipairs(self.rows) do
        if cellSizes[i] == nil then cellSizes[i] = {} end

        for columnName, widget in pairs(row) do
            widget:layout(maxSize, cv)
            cellSizes[i][columnName] = widget.size
        end
    end

    local columnWidths = {}
    local rowHeights = {}
    for rowIndex, row in ipairs(cellSizes) do
        for columnName, size in pairs(row) do
            if columnWidths[columnName] == nil or size.x > columnWidths[columnName] then
                columnWidths[columnName] = size.x
            end

            if rowHeights[rowIndex] == nil or size.y > rowHeights[rowIndex] then
                rowHeights[rowIndex] = size.y
            end
        end
    end

    for i, rowHeight in ipairs(rowHeights) do
        if rowHeight < self.minRowHeight then
            rowHeights[i] = self.minRowHeight
        end
    end

    self.size = Vector(0, 0)

    local columnCursors = {}

    self.cellInfo = {}

    for rowIndex, row in ipairs(self.rows) do
        local cursorX = 0
        for _, columnName in ipairs(self.columns) do
            local widget = row[columnName]
            local cursorY = columnCursors[columnName] or 0
            widget.pos = Vector(cursorX, cursorY)

            self.cellInfo[rowIndex] = self.cellInfo[rowIndex] or {}
            self.cellInfo[rowIndex][columnName] = {
                pos = widget.pos,
                size = Vector(columnWidths[columnName], rowHeights[rowIndex]),
            }

            cursorX = cursorX + columnWidths[columnName]
            cursorY = cursorY + rowHeights[rowIndex]
            columnCursors[columnName] = cursorY

            self.size.x = math.max(self.size.x, cursorX)
            self.size.y = math.max(self.size.y, cursorY)
        end
    end
end

function Table:paint(cv)
    local style = self.style
    local cellBorderWidth = style.cellBorderWidth
    local cellBorderColor = style.cellBorderColor
    local backgroundColor = style.backgroundColor

    -- Background
    cv:beginPath()
    cv:rect(Vector(0, 0), self.size)
    cv:solidColor(backgroundColor)
    cv:fill()

    self:paintChildren(cv)

    -- Cell borders
    for _, row in ipairs(self.cellInfo) do
        for _, info in pairs(row) do
            cv:beginPath()
            cv:rect(info.pos, info.size)
            cv:solidColor(cellBorderColor)
            cv:strokeWidth(cellBorderWidth)
            cv:stroke()
        end
    end
end

return Table
