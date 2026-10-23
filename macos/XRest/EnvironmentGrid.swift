import Foundation

/// Spreadsheet view of the workspace environments: one row per variable name (the union across
/// environments, in first-seen order), one column per environment.
///
/// A cell is absent when that environment does not define the variable; absent cells are not
/// written back. Pure value type so the conversion is unit tested without any UI.
struct EnvironmentGrid {
    struct Column: Equatable {
        var name: String
        var isUnsafe: Bool
    }

    struct Row: Identifiable {
        let id = UUID()
        var name: String
        var cells: [String: NameValue]
        /// Position of this variable within each environment's own list.
        var ranks: [String: Int] = [:]
    }

    private(set) var columns: [Column]
    private(set) var rows: [Row]

    init(environments: [EnvironmentConfig]) {
        columns = environments.map { Column(name: $0.name, isUnsafe: $0.isUnsafe) }
        var rows: [Row] = []
        for env in environments {
            for (index, v) in env.variables.enumerated() {
                // Reuse a row of the same name unless this environment already has a cell there
                // (a duplicate inside one environment gets its own row so nothing is dropped).
                if let i = rows.firstIndex(where: { $0.name == v.name && $0.cells[env.name] == nil }) {
                    rows[i].cells[env.name] = v
                    rows[i].ranks[env.name] = index
                } else {
                    rows.append(Row(name: v.name, cells: [env.name: v], ranks: [env.name: index]))
                }
            }
        }
        self.rows = rows
    }

    /// Variable names are trimmed; each environment keeps its own variable order.
    func toConfigs() -> [EnvironmentConfig] {
        columns.map { col in
            let ordered = rows.enumerated()
                .filter { $0.element.cells[col.name] != nil }
                .sorted {
                    let l = $0.element.ranks[col.name] ?? Int.max
                    let r = $1.element.ranks[col.name] ?? Int.max
                    return l != r ? l < r : $0.offset < $1.offset
                }
            return EnvironmentConfig(
                name: col.name, isUnsafe: col.isUnsafe,
                variables: ordered.map { _, row in
                    var cell = row.cells[col.name]!
                    cell.name = row.name.trimmingCharacters(in: .whitespaces)
                    return cell
                })
        }
    }

    func isDirty(comparedTo original: [EnvironmentConfig]) -> Bool {
        // Compare against the grid's own normalized form so loading never looks edited.
        toConfigs() != EnvironmentGrid(environments: original).toConfigs()
    }

    /// Problems that would make the core reject or mangle the save.
    var validationErrors: [String] {
        var errors: [String] = []
        var seen = Set<String>()
        var reported = Set<String>()
        for row in rows {
            let name = row.name.trimmingCharacters(in: .whitespaces)
            if name.isEmpty {
                if !errors.contains("A variable has no name.") { errors.append("A variable has no name.") }
            } else if !seen.insert(name).inserted, reported.insert(name).inserted {
                errors.append("Variable \(name) appears more than once.")
            }
        }
        var names = Set<String>()
        var reportedEnvs = Set<String>()
        for col in columns {
            let name = col.name.trimmingCharacters(in: .whitespaces)
            if name.isEmpty {
                if !errors.contains("An environment has no name.") { errors.append("An environment has no name.") }
            } else if !names.insert(name).inserted, reportedEnvs.insert(name).inserted {
                errors.append("Environment \(name) appears more than once.")
            }
        }
        if columns.isEmpty { errors.append("At least one environment is required.") }
        return errors
    }

    // MARK: Editing

    /// Append a safe, empty environment.
    mutating func addColumn(name: String) {
        columns.append(Column(name: name.trimmingCharacters(in: .whitespaces), isUnsafe: false))
    }

    /// Re-key the column's cells and ranks under the new name. Uniqueness is not enforced here
    /// (`validationErrors` reports it); if the new name is already taken, the existing
    /// environment's cells win so nothing of theirs is overwritten.
    mutating func renameColumn(from old: String, to new: String) {
        guard old != new, let i = columns.firstIndex(where: { $0.name == old }) else { return }
        columns[i].name = new
        let stillUsesOld = columns.contains { $0.name == old }
        for r in rows.indices {
            if let cell = rows[r].cells[old] {
                if rows[r].cells[new] == nil {
                    rows[r].cells[new] = cell
                    rows[r].ranks[new] = rows[r].ranks[old]
                }
                if !stillUsesOld {
                    rows[r].cells[old] = nil
                    rows[r].ranks[old] = nil
                }
            }
        }
    }

    mutating func removeColumn(name: String) {
        guard let i = columns.firstIndex(where: { $0.name == name }) else { return }
        columns.remove(at: i)
        guard !columns.contains(where: { $0.name == name }) else { return }
        for r in rows.indices {
            rows[r].cells[name] = nil
            rows[r].ranks[name] = nil
        }
    }

    mutating func moveColumn(from source: IndexSet, to destination: Int) {
        columns.move(fromOffsets: source, toOffset: destination)
    }

    mutating func addRow() {
        rows.append(Row(name: "", cells: [:]))
    }

    mutating func removeRow(id: UUID) {
        rows.removeAll { $0.id == id }
    }

    mutating func setName(rowID: UUID, _ name: String) {
        update(rowID) { $0.name = name }
    }

    mutating func setUnsafe(env: String, _ unsafe: Bool) {
        if let i = columns.firstIndex(where: { $0.name == env }) { columns[i].isUnsafe = unsafe }
    }

    mutating func setValue(rowID: UUID, env: String, _ value: String) {
        updateCell(rowID, env) { $0.value = value }
    }

    mutating func setEnabled(rowID: UUID, env: String, _ enabled: Bool) {
        updateCell(rowID, env) { $0.enabled = enabled }
    }

    /// Point the cell at a stored secret. The value is cleared; only the key is kept.
    mutating func linkSecret(rowID: UUID, env: String, key: String) {
        updateCell(rowID, env) {
            $0.secretKey = key
            $0.kind = "secret"
            $0.value = ""
        }
    }

    mutating func unlinkSecret(rowID: UUID, env: String) {
        updateCell(rowID, env) {
            $0.secretKey = nil
            $0.kind = "plain"
        }
    }

    // MARK: Helpers

    private mutating func update(_ id: UUID, _ change: (inout Row) -> Void) {
        if let i = rows.firstIndex(where: { $0.id == id }) { change(&rows[i]) }
    }

    private mutating func updateCell(_ id: UUID, _ env: String, _ change: (inout NameValue) -> Void) {
        let nextRank = (rows.compactMap { $0.ranks[env] }.max() ?? -1) + 1
        update(id) { row in
            var cell =
                row.cells[env]
                ?? NameValue(name: row.name, value: "", enabled: true, secretKey: nil, kind: "plain")
            change(&cell)
            if row.cells[env] == nil {
                row.ranks[env] = nextRank
            }
            row.cells[env] = cell
        }
    }
}
