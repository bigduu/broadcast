import{d as b,r as c,o as D,a as B,b as P,c as S,e,w as l,x as t,f as v,t as I,B as f,C as y,m as h,_ as z}from"./index-7d86bbc0.js";import{g as E,p as M,a as T}from"./api-c6756a56.js";const U=["src"],j=b({__name:"NodeDetail",props:{name:null,ip:null},setup(i){const n=i,o=c(""),a=c(!1),_=c(0);D(async()=>{r(),_.value=setInterval(async()=>{r()},1e3)}),B(()=>{clearInterval(_.value)});const r=()=>{E(n.ip).then(s=>{o.value=s,a.value=!1}).catch(s=>{o.value!==""&&(a.value=!0)})},w=async()=>{await M(n.ip)},x=async()=>{await T(n.ip)};return(s,q)=>{const u=t("el-divider"),d=t("el-col"),V=t("el-alert"),g=t("el-row"),k=t("VideoPlay"),p=t("el-icon"),m=t("el-button"),C=t("VideoPause"),N=t("el-card");return P(),S("div",null,[e(N,null,{default:l(()=>[e(g,{type:"flex"},{default:l(()=>[e(d,{style:{"text-align":"center"},class:"video-name"},{default:l(()=>[v("span",null,I(i.name),1),e(u)]),_:1}),e(d,null,{default:l(()=>[f(v("img",{style:{width:"100%",height:"50vh"},alt:"picture",src:o.value},null,8,U),[[y,!a.value]]),f(e(V,{style:{width:"100%"},title:"获取截图失败",type:"error"},null,512),[[y,a.value]])]),_:1})]),_:1}),e(u),e(m,{class:"inline-block",type:"primary",onClick:w,size:"small"},{default:l(()=>[e(p,null,{default:l(()=>[e(k)]),_:1}),h(" 播放 ")]),_:1}),e(m,{class:"inline-block",type:"primary",onClick:x,size:"small"},{default:l(()=>[e(p,null,{default:l(()=>[e(C)]),_:1}),h(" 暂停 ")]),_:1})]),_:1})])}}});const G=z(j,[["__scopeId","data-v-6076d760"]]);export{G as default};
