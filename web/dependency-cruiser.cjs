/** @type {import('dependency-cruiser').IConfiguration} */
module.exports = {
  forbidden: [
    {
      name: 'no-circular',
      severity: 'error',
      comment: 'Keep imports one-way. Extract shared contracts instead of importing back.',
      from: {},
      to: { circular: true },
    },
    {
      name: 'types-are-leaves',
      severity: 'error',
      comment: 'API types may only depend on other API types.',
      from: { path: '^src/types/' },
      to: { pathNot: '^src/types/' },
    },
    {
      name: 'lib-no-upward',
      severity: 'error',
      comment: 'lib is the lowest runtime layer. It cannot depend on UI or state.',
      from: { path: '^src/lib/' },
      to: { path: '^src/(store|components|features|pages|app)/' },
    },
    {
      name: 'store-no-ui',
      severity: 'error',
      comment: 'State must not depend on components, features, pages, or app composition.',
      from: { path: '^src/store/' },
      to: { path: '^src/(components|features|pages|app)/' },
    },
    {
      name: 'components-presentational',
      severity: 'error',
      comment: 'Components are presentational. State and feature behavior belong elsewhere.',
      from: { path: '^src/components/' },
      to: { path: '^src/(store|features|pages|app)/' },
    },
    {
      name: 'features-no-pages',
      severity: 'error',
      comment: 'Features must not depend on route screens or the app shell.',
      from: { path: '^src/features/' },
      to: { path: '^src/(pages|app)/' },
    },
    {
      name: 'pages-no-app',
      severity: 'error',
      comment: 'Pages are composed by app/. They must not import app composition code.',
      from: { path: '^src/pages/' },
      to: { path: '^src/app/' },
    },
  ],
  options: {
    doNotFollow: { path: 'node_modules' },
    exclude: { path: '(^|/)(dist|coverage|node_modules)/' },
    tsConfig: { fileName: 'tsconfig.json' },
    tsPreCompilationDeps: true,
  },
}
