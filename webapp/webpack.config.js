const path = require('path');


module.exports = {
    mode : 'development',

    resolve: {
        extensions: ['.ts', '.tsx', '.js']
    },

    entry: {
        app: './src/index.tsx'
    },

    module: {
        rules: [
            {
                test: /\.(ts|tsx)$/,
                loader: 'ts-loader',
                options: {
                    configFile: 'tsconfig.json'
                }
            }
        ],
    },

    output: {
        filename: '[name].bundle.js',
        path: path.resolve(path.join(__dirname,'../static/'))
    },

    devtool: 'eval-source-map'

};
