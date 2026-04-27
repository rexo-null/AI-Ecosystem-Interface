function Tabs({ openFiles, activeFile, onTabClick, onTabClose }) {
  const getLanguageIcon = (fileName) => {
    const ext = fileName.split('.').pop().toLowerCase()
    const icons = {
      js: 'JS',
      jsx: 'React',
      ts: 'TS',
      tsx: 'TSX',
      py: 'PY',
      html: '<>',
      css: '#',
      json: '{}',
      md: 'MD',
    }
    return icons[ext] || 'TXT'
  }

  return (
    <div className="tabs-container">
      {openFiles.map(filePath => {
        const fileName = filePath.split('/').pop()
        return (
          <div
            key={filePath}
            className={`tab ${activeFile === filePath ? 'active' : ''}`}
            onClick={() => onTabClick(filePath)}
          >
            <span>{getLanguageIcon(fileName)}</span>
            <span style={{ marginLeft: '6px' }}>{fileName}</span>
            <span 
              className="tab-close"
              onClick={(e) => {
                e.stopPropagation()
                onTabClose(filePath)
              }}
            >
              ×
            </span>
          </div>
        )
      })}
    </div>
  )
}

export default Tabs
