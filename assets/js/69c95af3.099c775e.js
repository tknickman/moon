"use strict";(self.webpackChunkwebsite=self.webpackChunkwebsite||[]).push([[7363],{5318:(e,t,n)=>{n.d(t,{Zo:()=>m,kt:()=>d});var o=n(7378);function r(e,t,n){return t in e?Object.defineProperty(e,t,{value:n,enumerable:!0,configurable:!0,writable:!0}):e[t]=n,e}function c(e,t){var n=Object.keys(e);if(Object.getOwnPropertySymbols){var o=Object.getOwnPropertySymbols(e);t&&(o=o.filter((function(t){return Object.getOwnPropertyDescriptor(e,t).enumerable}))),n.push.apply(n,o)}return n}function a(e){for(var t=1;t<arguments.length;t++){var n=null!=arguments[t]?arguments[t]:{};t%2?c(Object(n),!0).forEach((function(t){r(e,t,n[t])})):Object.getOwnPropertyDescriptors?Object.defineProperties(e,Object.getOwnPropertyDescriptors(n)):c(Object(n)).forEach((function(t){Object.defineProperty(e,t,Object.getOwnPropertyDescriptor(n,t))}))}return e}function l(e,t){if(null==e)return{};var n,o,r=function(e,t){if(null==e)return{};var n,o,r={},c=Object.keys(e);for(o=0;o<c.length;o++)n=c[o],t.indexOf(n)>=0||(r[n]=e[n]);return r}(e,t);if(Object.getOwnPropertySymbols){var c=Object.getOwnPropertySymbols(e);for(o=0;o<c.length;o++)n=c[o],t.indexOf(n)>=0||Object.prototype.propertyIsEnumerable.call(e,n)&&(r[n]=e[n])}return r}var i=o.createContext({}),s=function(e){var t=o.useContext(i),n=t;return e&&(n="function"==typeof e?e(t):a(a({},t),e)),n},m=function(e){var t=s(e.components);return o.createElement(i.Provider,{value:t},e.children)},p={inlineCode:"code",wrapper:function(e){var t=e.children;return o.createElement(o.Fragment,{},t)}},u=o.forwardRef((function(e,t){var n=e.components,r=e.mdxType,c=e.originalType,i=e.parentName,m=l(e,["components","mdxType","originalType","parentName"]),u=s(n),d=r,f=u["".concat(i,".").concat(d)]||u[d]||p[d]||c;return n?o.createElement(f,a(a({ref:t},m),{},{components:n})):o.createElement(f,a({ref:t},m))}));function d(e,t){var n=arguments,r=t&&t.mdxType;if("string"==typeof e||r){var c=n.length,a=new Array(c);a[0]=u;var l={};for(var i in t)hasOwnProperty.call(t,i)&&(l[i]=t[i]);l.originalType=e,l.mdxType="string"==typeof e?e:r,a[1]=l;for(var s=2;s<c;s++)a[s]=n[s];return o.createElement.apply(null,a)}return o.createElement.apply(null,n)}u.displayName="MDXCreateElement"},8819:(e,t,n)=>{n.r(t),n.d(t,{assets:()=>i,contentTitle:()=>a,default:()=>p,frontMatter:()=>c,metadata:()=>l,toc:()=>s});var o=n(5773),r=(n(7378),n(5318));const c={title:"completions"},a=void 0,l={unversionedId:"commands/completions",id:"commands/completions",title:"completions",description:"The moon completions command will generate moon command and argument completions for your current",source:"@site/docs/commands/completions.mdx",sourceDirName:"commands",slug:"/commands/completions",permalink:"/docs/commands/completions",draft:!1,editUrl:"https://github.com/moonrepo/moon/tree/master/website/docs/commands/completions.mdx",tags:[],version:"current",frontMatter:{title:"completions"},sidebar:"docs",previous:{title:"clean",permalink:"/docs/commands/clean"},next:{title:"dep-graph",permalink:"/docs/commands/dep-graph"}},i={},s=[{value:"Options",id:"options",level:3}],m={toc:s};function p(e){let{components:t,...n}=e;return(0,r.kt)("wrapper",(0,o.Z)({},m,n,{components:t,mdxType:"MDXLayout"}),(0,r.kt)("p",null,"The ",(0,r.kt)("inlineCode",{parentName:"p"},"moon completions")," command will generate moon command and argument completions for your current\nshell. This command will write to stdout, which can then be redirected to a file of your choice."),(0,r.kt)("pre",null,(0,r.kt)("code",{parentName:"pre",className:"language-shell"},"$ moon completions > ~/.bash_completion.d/moon.sh\n")),(0,r.kt)("h3",{id:"options"},"Options"),(0,r.kt)("ul",null,(0,r.kt)("li",{parentName:"ul"},(0,r.kt)("inlineCode",{parentName:"li"},"--shell"),' - Shell to explicitly generate for. Accepts "bash", "elvish", "fish", "powershell", or\n"zsh".')))}p.isMDXComponent=!0}}]);